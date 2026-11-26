package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/joho/godotenv"
	"github.com/nguyenvanduocit/tanca-mcp/services"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}
	switch os.Args[1] {
	case "list-employees":
		runListEmployees(os.Args[2:])
	case "list-timekeeping-employees":
		runListTimekeepingEmployees(os.Args[2:])
	case "list-shifts":
		runListShifts(os.Args[2:])
	case "get-shift-summary":
		runGetShiftSummary(os.Args[2:])
	case "get-today-shift":
		runGetTodayShift(os.Args[2:])
	case "get-shift-by-day":
		runGetShiftByDay(os.Args[2:])
	case "check-in":
		runCheckIn(os.Args[2:])
	case "check-out":
		runCheckOut(os.Args[2:])
	case "get-clock-logs":
		runGetClockLogs(os.Args[2:])
	case "get-shop-info":
		runGetShopInfo(os.Args[2:])
	case "make-shift-change-request":
		runMakeShiftChangeRequest(os.Args[2:])
	case "list-requests":
		runListRequests(os.Args[2:])
	case "get-request-detail":
		runGetRequestDetail(os.Args[2:])
	case "help", "--help", "-h":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("tanca-cli - Tanca API command line interface")
	fmt.Println()
	fmt.Println("Usage:")
	fmt.Println("  tanca-cli <command> [flags]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  list-employees               List all employees")
	fmt.Println("  list-timekeeping-employees   List employees requiring timekeeping")
	fmt.Println("  list-shifts                  List all available shift assignments")
	fmt.Println("  get-shift-summary            Get shift summary for an employee")
	fmt.Println("  get-today-shift              Get today's shift for an employee")
	fmt.Println("  get-shift-by-day             Get an employee's shift for a specific day (YYYY-MM-DD)")
	fmt.Println("  check-in                     Record a check-in")
	fmt.Println("  check-out                    Record a check-out")
	fmt.Println("  get-clock-logs               Get clock logs for a shift")
	fmt.Println("  get-shop-info                Get company/shop information")
	fmt.Println("  make-shift-change-request    Submit a time-change request (đề xuất chỉnh giờ vào/ra)")
	fmt.Println("  list-requests                List submitted requests filtered by date/type/status")
	fmt.Println("  get-request-detail           Get full details of a request by ID")
	fmt.Println()
	fmt.Println("Global flags (available on all commands):")
	fmt.Println("  --env string     Path to .env file")
	fmt.Println("  --output string  Output format: text (default) or json")
}

func loadEnvAndClient(envFile string) *services.Client {
	if envFile != "" {
		if err := godotenv.Load(envFile); err != nil {
			fmt.Fprintf(os.Stderr, "warning: failed to load env file %s: %v\n", envFile, err)
		}
	}
	return services.TancaClient()
}

func printJSON(v interface{}) {
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err := enc.Encode(v); err != nil {
		fmt.Fprintf(os.Stderr, "json encode error: %v\n", err)
		os.Exit(1)
	}
}

func runListEmployees(args []string) {
	fs := flag.NewFlagSet("list-employees", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	page := fs.Int("page", 0, "Page number for pagination")
	limit := fs.Int("limit", 0, "Number of employees per page (max 100)")
	departmentID := fs.String("department-id", "", "Filter by department ID")
	startDate := fs.String("start-date", "", "Filter start date (YYYY-MM-DD)")
	endDate := fs.String("end-date", "", "Filter end date (YYYY-MM-DD)")
	fs.Parse(args)

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	params := services.GetEmployeeListParams{
		Page:      *page,
		Limit:     *limit,
		StartDate: *startDate,
		EndDate:   *endDate,
	}
	if *departmentID != "" {
		params.DepartmentIDs = []string{*departmentID}
	}

	data, err := client.GetEmployeeList(ctx, params)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(data)
		return
	}

	fmt.Printf("Employees (Page %d/%d, Total: %d)\n\n",
		data.Meta.CurrentPage, data.Meta.TotalPages, data.Meta.Total)
	for _, emp := range data.Items {
		timekeeping := "Yes"
		if emp.IsNoNeedTimekeeping == 1 {
			timekeeping = "No"
		}
		fmt.Printf("- %s (%s)\n", emp.Name, emp.Identification)
		fmt.Printf("  User ID: %s\n", emp.UserID)
		fmt.Printf("  Phone: %s\n", emp.Username)
		fmt.Printf("  Branch ID: %s\n", emp.BranchID)
		fmt.Printf("  Department ID: %s\n", emp.DepartmentID)
		fmt.Printf("  Position ID: %s\n", emp.PositionID)
		fmt.Printf("  Needs Timekeeping: %s\n", timekeeping)
		if emp.WorkingDate != "" {
			fmt.Printf("  Working Date: %s\n", emp.WorkingDate)
		}
		fmt.Println()
	}
}

func runListTimekeepingEmployees(args []string) {
	fs := flag.NewFlagSet("list-timekeeping-employees", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	fs.Parse(args)

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	employees, err := client.GetTimekeepingEmployees(ctx)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(employees)
		return
	}

	fmt.Printf("Employees Requiring Timekeeping (Total: %d)\n\n", len(employees))
	for _, emp := range employees {
		fmt.Printf("- %s (%s)\n", emp.Name, emp.Identification)
		fmt.Printf("  User ID: %s\n", emp.UserID)
		fmt.Printf("  Phone: %s\n", emp.Username)
		fmt.Printf("  Branch ID: %s\n", emp.BranchID)
		fmt.Printf("  Department ID: %s\n", emp.DepartmentID)
		fmt.Println()
	}
}

func runListShifts(args []string) {
	fs := flag.NewFlagSet("list-shifts", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	fs.Parse(args)

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	shifts, err := client.GetShiftAssignments(ctx)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(shifts)
		return
	}

	fmt.Printf("Available Shifts (Total: %d)\n\n", len(shifts))
	for _, sa := range shifts {
		fmt.Printf("- %s (ID: %s)\n", sa.Title, sa.ID)
		fmt.Printf("  Shift: %s (%s)\n", sa.Shift.Name, sa.Shift.ShiftKey)
		fmt.Printf("  Time: %s:%s - %s:%s\n",
			sa.Shift.StartHourObj.Value, sa.Shift.StartMinuteObj.Value,
			sa.Shift.EndHourObj.Value, sa.Shift.EndMinuteObj.Value)
		fmt.Printf("  Working Hours: %d\n", sa.Shift.WorkingHour)
		if sa.Shift.IsOvertimeShift == 1 {
			fmt.Println("  Type: Overtime Shift")
		}
		fmt.Println()
	}
}

func runGetShiftSummary(args []string) {
	fs := flag.NewFlagSet("get-shift-summary", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	startDate := fs.String("start-date", "", "Start date (YYYY-MM-DD) (required)")
	endDate := fs.String("end-date", "", "End date (YYYY-MM-DD) (required)")
	fs.Parse(args)

	if *userID == "" || *startDate == "" || *endDate == "" {
		fmt.Fprintf(os.Stderr, "error: --user-id, --start-date, and --end-date are required\n")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	summary, err := client.GetEmployeeShiftSummary(ctx, services.GetEmployeeShiftSummaryParams{
		UserID:    *userID,
		StartDate: *startDate,
		EndDate:   *endDate,
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(summary)
		return
	}

	fmt.Printf("Shift Summary for %s (%s)\n", summary.Name, summary.Identification)
	fmt.Printf("Phone: %s\n", summary.Phone)
	fmt.Printf("Total Working Hours: %d\n", summary.TotalWorkingHour)
	fmt.Printf("Real Working Hours: %d\n", summary.RealWorkingHour)
	fmt.Println("\nShifts:")
	for date, shifts := range summary.Shifts {
		fmt.Printf("\n%s:\n", date)
		for _, shift := range shifts {
			fmt.Printf("  - %s (ID: %s)\n", shift.Name, shift.ID)
			fmt.Printf("    Time: %s - %s\n", shift.StartTime, shift.EndTime)
			fmt.Printf("    Working Hours: %d\n", shift.WorkingHour)
			if shift.CheckinTime != nil {
				fmt.Printf("    Check-in: %s (%s)\n", *shift.CheckinTime, shift.CheckinType)
			} else {
				fmt.Println("    Check-in: Not recorded")
			}
			if shift.CheckoutTime != nil {
				fmt.Printf("    Check-out: %s (%s)\n", *shift.CheckoutTime, shift.CheckoutType)
			} else {
				fmt.Println("    Check-out: Not recorded")
			}
			if shift.Status != nil {
				fmt.Printf("    Status: %s\n", shift.Status.Name)
			}
		}
	}
}

func runGetTodayShift(args []string) {
	fs := flag.NewFlagSet("get-today-shift", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	fs.Parse(args)

	if *userID == "" {
		fmt.Fprintf(os.Stderr, "error: --user-id is required\n")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	shift, err := client.GetTodayShift(ctx, *userID)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(shift)
		return
	}

	fmt.Printf("Today's Shift for User %s\n\n", *userID)
	fmt.Printf("Shift Assignment ID: %s\n", shift.ID)
	fmt.Printf("Shift Name: %s\n", shift.Name)
	fmt.Printf("Shift Key: %s\n", shift.ShiftKey)
	fmt.Printf("Time: %s - %s\n", shift.StartTime, shift.EndTime)
	fmt.Printf("Working Day: %s\n", shift.WorkingDay)
	fmt.Printf("Working Hours: %d\n", shift.WorkingHour)
	fmt.Printf("Branch ID: %s\n", shift.BranchID)
	if shift.CheckinTime != nil {
		fmt.Printf("\nCheck-in: %s (%s)\n", *shift.CheckinTime, shift.CheckinType)
	} else {
		fmt.Println("\nCheck-in: Not recorded")
	}
	if shift.CheckoutTime != nil {
		fmt.Printf("Check-out: %s (%s)\n", *shift.CheckoutTime, shift.CheckoutType)
	} else {
		fmt.Println("Check-out: Not recorded")
	}
	if shift.Status != nil {
		fmt.Printf("\nStatus: %s\n", shift.Status.Name)
	}
}

func runGetShiftByDay(args []string) {
	fs := flag.NewFlagSet("get-shift-by-day", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	workingDay := fs.String("working-day", "", "Working day YYYY-MM-DD (required)")
	fs.Parse(args)

	if *userID == "" || *workingDay == "" {
		fmt.Fprintln(os.Stderr, "error: --user-id and --working-day are required")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	shifts, err := client.ListEmployeeShiftsForDay(ctx, *userID, *workingDay)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(shifts)
		return
	}

	fmt.Printf("Shifts for User %s on %s (Total: %d)\n\n", *userID, *workingDay, len(shifts))
	if len(shifts) == 0 {
		fmt.Println("No shift assigned for this day.")
		return
	}
	for _, shift := range shifts {
		fmt.Printf("- %s (%s)\n", shift.Name, shift.ShiftKey)
		fmt.Printf("  Employee Shift ID: %s\n", shift.ID)
		fmt.Printf("  Time: %s → %s\n", shift.StartTime, shift.EndTime)
		fmt.Printf("  Working Hours: %d\n", shift.WorkingHour)
		fmt.Printf("  Branch ID: %s\n", shift.BranchID)
		if shift.CheckinTime != nil {
			fmt.Printf("  Check-in: %s\n", *shift.CheckinTime)
		} else {
			fmt.Println("  Check-in: Not recorded")
		}
		if shift.CheckoutTime != nil {
			fmt.Printf("  Check-out: %s\n", *shift.CheckoutTime)
		} else {
			fmt.Println("  Check-out: Not recorded")
		}
		fmt.Println()
	}
}

func runCheckIn(args []string) {
	fs := flag.NewFlagSet("check-in", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	shiftAssignmentID := fs.String("shift-assignment-id", "", "Shift assignment ID (required)")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	branchID := fs.String("branch-id", "", "Branch ID (required)")
	checkinTime := fs.String("checkin-time", "", "Check-in time (YYYY-MM-DD HH:mm:ss). Defaults to now.")
	fs.Parse(args)

	if *shiftAssignmentID == "" || *userID == "" || *branchID == "" {
		fmt.Fprintf(os.Stderr, "error: --shift-assignment-id, --user-id, and --branch-id are required\n")
		os.Exit(1)
	}

	var t time.Time
	if *checkinTime != "" {
		loc, _ := time.LoadLocation("Asia/Saigon")
		var err error
		t, err = time.ParseInLocation(services.TancaTimeFormat, *checkinTime, loc)
		if err != nil {
			fmt.Fprintf(os.Stderr, "error: invalid --checkin-time format (use YYYY-MM-DD HH:mm:ss): %v\n", err)
			os.Exit(1)
		}
	} else {
		t = time.Now()
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	resp, err := client.CheckIn(ctx, *shiftAssignmentID, *userID, *branchID, t)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(resp)
		return
	}

	fmt.Println("Check-in recorded successfully!")
	fmt.Println()
	fmt.Printf("Shift: %s\n", resp.Name)
	fmt.Printf("Check-in Time: %s\n", resp.CheckinTime)
	fmt.Printf("Check-in Type: %s\n", resp.CheckinType)
	if resp.Status != nil {
		fmt.Printf("Status: %s\n", resp.Status.Name)
	}
}

func runCheckOut(args []string) {
	fs := flag.NewFlagSet("check-out", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	shiftAssignmentID := fs.String("shift-assignment-id", "", "Shift assignment ID (required)")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	branchID := fs.String("branch-id", "", "Branch ID (required)")
	checkoutTime := fs.String("checkout-time", "", "Check-out time (YYYY-MM-DD HH:mm:ss). Defaults to now.")
	fs.Parse(args)

	if *shiftAssignmentID == "" || *userID == "" || *branchID == "" {
		fmt.Fprintf(os.Stderr, "error: --shift-assignment-id, --user-id, and --branch-id are required\n")
		os.Exit(1)
	}

	var t time.Time
	if *checkoutTime != "" {
		loc, _ := time.LoadLocation("Asia/Saigon")
		var err error
		t, err = time.ParseInLocation(services.TancaTimeFormat, *checkoutTime, loc)
		if err != nil {
			fmt.Fprintf(os.Stderr, "error: invalid --checkout-time format (use YYYY-MM-DD HH:mm:ss): %v\n", err)
			os.Exit(1)
		}
	} else {
		t = time.Now()
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	resp, err := client.CheckOut(ctx, *shiftAssignmentID, *userID, *branchID, t)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(resp)
		return
	}

	fmt.Println("Check-out recorded successfully!")
	fmt.Println()
	fmt.Printf("Shift: %s\n", resp.Name)
	fmt.Printf("Check-out Time: %s\n", resp.CheckoutTime)
	fmt.Printf("Check-out Type: %s\n", resp.CheckoutType)
	if resp.Status != nil {
		fmt.Printf("Status: %s\n", resp.Status.Name)
	}
}

func runGetClockLogs(args []string) {
	fs := flag.NewFlagSet("get-clock-logs", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	employeeShiftID := fs.String("employee-shift-id", "", "Employee shift ID (required)")
	fs.Parse(args)

	if *employeeShiftID == "" {
		fmt.Fprintf(os.Stderr, "error: --employee-shift-id is required\n")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	logs, err := client.GetCheckedTime(ctx, *employeeShiftID)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(logs)
		return
	}

	fmt.Printf("Clock Logs for Shift %s (Total: %d)\n\n", *employeeShiftID, len(logs))
	for i, log := range logs {
		trashed := ""
		if log.IsTrashed == 1 {
			trashed = " [DELETED]"
		}
		fmt.Printf("%d. %s%s\n", i+1, log.Time, trashed)
		fmt.Printf("   Type: %s\n", log.ClockType)
		fmt.Printf("   Created: %s\n", log.CreatedAt)
		if log.CreatedUser != nil {
			fmt.Printf("   Created By: %s\n", log.CreatedUser.Name)
		}
		if log.Reason != "" {
			fmt.Printf("   Reason: %s\n", log.Reason)
		}
		if log.Location != nil {
			fmt.Printf("   Location: %s, %s\n", log.Location.Latitude, log.Location.Longitude)
		}
		fmt.Println()
	}
}

func runGetShopInfo(args []string) {
	fs := flag.NewFlagSet("get-shop-info", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	fs.Parse(args)

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	shop, err := client.GetShopDetail(ctx)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(shop)
		return
	}

	fmt.Println("=== Company Information ===")
	fmt.Println()
	fmt.Printf("Shop ID: %s\n", shop.ID)
	fmt.Printf("Name: %s\n", shop.Name)
	fmt.Printf("Username: %s\n", shop.Username)
	if shop.Email != "" {
		fmt.Printf("Email: %s\n", shop.Email)
	}
	if shop.Phone != "" {
		fmt.Printf("Phone: %s\n", shop.Phone)
	}
	if shop.Website != "" {
		fmt.Printf("Website: %s\n", shop.Website)
	}
	if shop.Address != "" {
		fmt.Printf("Address: %s\n", shop.Address)
	}
	if shop.Country != nil {
		fmt.Printf("Country: %s\n", shop.Country.Name)
	}
	if shop.Province != nil {
		fmt.Printf("Province: %s\n", shop.Province.Name)
	}
	if shop.District != nil {
		fmt.Printf("District: %s\n", shop.District.Name)
	}
	if shop.TaxCode != "" {
		fmt.Printf("Tax Code: %s\n", shop.TaxCode)
	}
	if shop.FoundedDate != "" {
		fmt.Printf("Founded: %s\n", shop.FoundedDate)
	}
	fmt.Printf("Created At: %s\n", shop.CreatedAt)
	if shop.PackagesInfo != nil {
		fmt.Println("\n=== Subscription ===")
		info := shop.PackagesInfo.Info
		fmt.Printf("Period: %s to %s\n", info.StartPackage, info.EndPackage)
		fmt.Printf("Users: %d active / %d total\n", info.ActiveUsers, info.Users)
		if len(shop.PackagesInfo.Packages) > 0 {
			fmt.Println("\nActive Packages:")
			for _, pkg := range shop.PackagesInfo.Packages {
				if pkg.IsAvailable == 1 {
					fmt.Printf("  - %s (%s)\n", pkg.Name, pkg.Code)
				}
			}
		}
	}
}

func runMakeShiftChangeRequest(args []string) {
	fs := flag.NewFlagSet("make-shift-change-request", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	userID := fs.String("user-id", "", "Employee's user ID (required)")
	employeeShiftID := fs.String("employee-shift-id", "", "Employee shift ID for the working day (required)")
	workingDay := fs.String("working-day", "", "Working day YYYY-MM-DD (required)")
	checkinTime := fs.String("checkin-time", "", "Proposed check-in datetime YYYY-MM-DD HH:mm:ss (required)")
	checkoutTime := fs.String("checkout-time", "", "Proposed check-out datetime YYYY-MM-DD HH:mm:ss (required)")
	reason := fs.String("reason", "", "Reason for the change (required)")
	fs.Parse(args)

	if *userID == "" || *employeeShiftID == "" || *workingDay == "" || *checkinTime == "" || *checkoutTime == "" || *reason == "" {
		fmt.Fprintln(os.Stderr, "error: --user-id, --employee-shift-id, --working-day, --checkin-time, --checkout-time, and --reason are required")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	req, err := client.MakeShiftChangeRequest(ctx, services.MakeShiftChangeRequestParams{
		UserID:          *userID,
		EmployeeShiftID: *employeeShiftID,
		WorkingDay:      *workingDay,
		CheckinTime:     *checkinTime,
		CheckoutTime:    *checkoutTime,
		Reason:          *reason,
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(req)
		return
	}

	fmt.Println("Time change request submitted successfully!")
	fmt.Println()
	fmt.Printf("Request ID: %s\n", req.ID)
	fmt.Printf("Serial: %s\n", req.Serial)
	fmt.Printf("Status: %s\n", requestStatusName(req.Status))
	fmt.Printf("Check-in: %s\n", req.CheckinDatetime)
	fmt.Printf("Check-out: %s\n", req.CheckoutDatetime)
	fmt.Printf("Reason: %s\n", req.Reason)
	if req.UserObj != nil {
		fmt.Printf("Employee: %s (%s)\n", req.UserObj.Name, req.UserObj.Username)
	}
	if len(req.ApproverObj) > 0 {
		names := make([]string, 0, len(req.ApproverObj))
		for _, a := range req.ApproverObj {
			names = append(names, a.Name)
		}
		fmt.Printf("Approver(s): %s\n", strings.Join(names, ", "))
	}
}

func runListRequests(args []string) {
	fs := flag.NewFlagSet("list-requests", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	fromDate := fs.String("from-date", "", "Start date YYYY-MM-DD (required)")
	toDate := fs.String("to-date", "", "End date YYYY-MM-DD (required)")
	reqType := fs.String("type", "", "Type filter: employee-shift, onleave, overtime, etc.")
	statusFlag := fs.Int("status", -1, "Status filter: 0=pending, 1=approved, 2=rejected. -1 for all.")
	page := fs.Int("page", 1, "Page number")
	fs.Parse(args)

	if *fromDate == "" || *toDate == "" {
		fmt.Fprintln(os.Stderr, "error: --from-date and --to-date are required")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	params := services.ListRequestsParams{
		FromDate: *fromDate,
		ToDate:   *toDate,
		Type:     *reqType,
		Page:     *page,
	}
	if *statusFlag >= 0 {
		s := *statusFlag
		params.Status = &s
	}

	data, err := client.ListRequests(ctx, params)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(data)
		return
	}

	fmt.Printf("Requests %s → %s (Page %d/%d, Total: %d)\n\n",
		*fromDate, *toDate, data.Meta.CurrentPage, data.Meta.TotalPages, data.Meta.Total)
	for _, r := range data.Items {
		fmt.Printf("- [%s] %s — %s\n", r.Serial, requestTypeLabel(r.Type), requestStatusName(r.Status))
		fmt.Printf("  ID: %s\n", r.ID)
		if r.UserObj != nil {
			fmt.Printf("  Employee: %s\n", r.UserObj.Name)
		}
		if r.Reason != "" {
			fmt.Printf("  Reason: %s\n", r.Reason)
		}
		switch r.Type {
		case "employee-shift":
			if r.CheckinDatetime != "" {
				fmt.Printf("  Proposed: %s → %s\n", r.CheckinDatetime, r.CheckoutDatetime)
			}
		case "onleave":
			if r.FromDate != "" {
				fmt.Printf("  Range: %s → %s\n", r.FromDate, r.ToDate)
			}
		}
		fmt.Printf("  Created: %s\n\n", r.CreatedAt)
	}
}

func runGetRequestDetail(args []string) {
	fs := flag.NewFlagSet("get-request-detail", flag.ExitOnError)
	envFile := fs.String("env", "", "Path to .env file")
	output := fs.String("output", "text", "Output format: text or json")
	requestID := fs.String("request-id", "", "Request ID (required)")
	fs.Parse(args)

	if *requestID == "" {
		fmt.Fprintln(os.Stderr, "error: --request-id is required")
		os.Exit(1)
	}

	client := loadEnvAndClient(*envFile)
	ctx := context.Background()

	r, err := client.GetRequestDetail(ctx, *requestID)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if *output == "json" {
		printJSON(r)
		return
	}

	fmt.Printf("Request %s — %s\n\n", r.Serial, requestTypeLabel(r.Type))
	fmt.Printf("ID: %s\n", r.ID)
	fmt.Printf("Status: %s\n", requestStatusName(r.Status))
	fmt.Printf("Created: %s\n", r.CreatedAt)
	if r.UserObj != nil {
		fmt.Printf("Employee: %s (%s)\n", r.UserObj.Name, r.UserObj.Username)
	}
	if r.BranchObj != nil {
		fmt.Printf("Branch: %s\n", r.BranchObj.Name)
	}
	if r.DepartmentObj != nil {
		fmt.Printf("Department: %s\n", r.DepartmentObj.Name)
	}
	if r.Reason != "" {
		fmt.Printf("Reason: %s\n", r.Reason)
	}
	switch r.Type {
	case "employee-shift":
		if r.CheckinDatetime != "" {
			fmt.Printf("Proposed Check-in: %s\n", r.CheckinDatetime)
			fmt.Printf("Proposed Check-out: %s\n", r.CheckoutDatetime)
		}
	case "onleave":
		if r.FromDate != "" {
			fmt.Printf("From: %s\n", r.FromDate)
			fmt.Printf("To: %s\n", r.ToDate)
		}
	}
}

func requestStatusName(status int) string {
	switch status {
	case 0:
		return "Pending"
	case 1:
		return "Approved"
	case 2:
		return "Rejected"
	default:
		return fmt.Sprintf("Unknown (%d)", status)
	}
}

func requestTypeLabel(t string) string {
	switch t {
	case "employee-shift":
		return "Thay đổi giờ vào/ra"
	case "onleave":
		return "Nghỉ phép"
	case "overtime":
		return "Làm thêm giờ"
	case "business-trip":
		return "Công tác/Ra ngoài"
	case "register-shift":
		return "Đăng ký ca làm"
	case "company-advance", "advance-reimbursement", "reimbursement":
		return "Tạm ứng - Hoàn ứng"
	case "payment":
		return "Thanh toán"
	default:
		return t
	}
}
