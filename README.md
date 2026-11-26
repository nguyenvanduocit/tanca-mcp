# Tanca MCP và CLI

Tra cứu nhân viên, lịch ca, nhật ký chấm công và đề xuất trên Tanca bằng dòng lệnh hoặc qua một ứng dụng hỗ trợ Model Context Protocol (MCP). Cả hai dùng API v4 và quyền của tài khoản cung cấp token. Ba thao tác ghi là chấm vào, chấm ra và gửi đề xuất chỉnh giờ.

## Cài đặt từ mã nguồn

Cần Git và Rust bản stable (cài qua [rustup](https://rustup.rs)). Chạy các lệnh sau trong terminal trên macOS hoặc Linux.

```sh
git clone https://github.com/nguyenvanduocit/tanca-mcp.git
cd tanca-mcp
cargo build --release
mkdir -p bin
cp target/release/tanca-cli target/release/tanca-mcp bin/
./bin/tanca-cli help
```

## Lấy token từ phiên Tanca của bạn

1. Đăng nhập Tanca trên trình duyệt bằng tài khoản bạn được phép sử dụng. Trong Chrome hoặc Edge, mở Developer Tools và chọn **Network**.
2. Chọn **Fetch/XHR**, tải lại trang, rồi mở một màn hình bạn có quyền xem, chẳng hạn danh sách ca. Lọc theo `api.tanca.io`.
3. Chọn một yêu cầu thành công tới `https://api.tanca.io/api/v4/…`. Trong **Headers → Request Headers**, tìm `authorization`. Nếu đang xem yêu cầu `OPTIONS`, chọn yêu cầu dữ liệu tiếp theo.
4. Sao chép riêng giá trị header, gồm cả tiền tố `Bearer `. Tạo tệp cấu hình bằng các lệnh bên dưới, rồi dùng trình soạn thảo để dán giá trị vào giữa hai dấu nháy của `TANCA_TOKEN`. Chỉ lưu token trên máy bạn. Khi chia sẻ lỗi, hãy bỏ token và dữ liệu nhân sự khỏi ảnh hoặc log.

```sh
cp .env.example .env
chmod 600 .env
```

| Biến | Cách dùng |
| --- | --- |
| `TANCA_TOKEN` | Bắt buộc. Giá trị đầy đủ của header `authorization`. |
| `TANCA_SHOP_ID` | Tùy chọn. Client tra ID shop qua API khi cần. |
| `TANCA_BRANCH_ID` | Tùy chọn. ID chi nhánh dùng ở các thao tác hỗ trợ. |
| `TANCA_DEVICE` | Tùy chọn. Nếu phiên của bạn cần header `device`, dùng giá trị từ cùng yêu cầu. |

Truyền `--env .env` để đọc tệp. Nếu đã đặt biến môi trường cho tiến trình, có thể bỏ cờ này. Biến đã có trong môi trường được ưu tiên. Kiểm tra token bằng một lệnh chỉ đọc.

```sh
./bin/tanca-cli get-shop-info --env .env
```

## Dùng CLI

Đặt cờ sau tên lệnh. Dùng `--output json` để lấy dữ liệu có cấu trúc và `--help` sau từng lệnh để xem tham số, định dạng và giá trị mặc định.

```sh
./bin/tanca-cli list-employees --env .env --limit 5 --output json
./bin/tanca-cli get-today-shift --env .env --user-id USER_ID
./bin/tanca-cli make-shift-change-request --help
```

Thay `USER_ID` bằng ID từ danh sách nhân viên. Để ghi nhận chấm công, lấy ID ca được phân cho nhân viên từ kết quả tra ca theo ngày. Kiểm tra đúng người, chi nhánh, ca và thời gian trước khi chạy lệnh ghi.

| Lệnh | Chức năng |
| --- | --- |
| `get-shop-info` | Xem thông tin shop. |
| `list-employees` | Liệt kê nhân viên, hỗ trợ phân trang và lọc theo phòng ban. |
| `list-timekeeping-employees` | Xem danh sách nhân viên cần chấm công. |
| `list-shifts` | Liệt kê các ca được cấu hình. |
| `get-shift-summary` | Tổng hợp ca theo nhân viên và khoảng ngày. |
| `get-today-shift` / `get-shift-by-day` | Tra ca hôm nay hoặc ngày cụ thể của nhân viên. |
| `get-clock-logs` | Xem nhật ký chấm công theo ID ca của nhân viên. |
| `check-in` / `check-out` | Ghi giờ vào hoặc ra. Nếu bỏ trống giờ, CLI dùng giờ hiện tại. |
| `make-shift-change-request` | Gửi đề xuất chỉnh giờ vào và ra kèm lý do. |
| `list-requests` | Tra đề xuất theo khoảng ngày, loại và trạng thái. |
| `get-request-detail` | Xem chi tiết một đề xuất theo ID. |

## Kết nối ứng dụng MCP

**Stdio** là chế độ mặc định. Với ứng dụng nhận cấu hình `mcpServers`, thêm mục sau rồi khởi động lại kết nối. Thay hai chuỗi viết hoa bằng đường dẫn tuyệt đối tới binary và tệp môi trường trên máy bạn.

```json
{
  "mcpServers": {
    "tanca": {
      "command": "ABSOLUTE_PATH_TO_TANCA_MCP",
      "args": ["--env", "ABSOLUTE_PATH_TO_ENV_FILE"]
    }
  }
}
```

Mỗi lệnh CLI trong bảng có một tool MCP tương ứng. Đổi dấu gạch nối thành gạch dưới rồi thêm tiền tố `tanca_`, ví dụ `tanca_get_shop_info` và `tanca_get_today_shift`. Client có thể lấy toàn bộ tên và schema tham số bằng `tools/list`.

**Streamable HTTP** dùng endpoint `/mcp`. Chạy lệnh sau và cấu hình URL của client thành `http://127.0.0.1:8080/mcp`.

```sh
./bin/tanca-mcp --env .env --http_port 8080
```

Chế độ HTTP lắng nghe trên mọi interface và dùng chung token Tanca cho các client kết nối. Chỉ bật trong mạng được kiểm soát, với firewall giới hạn truy cập. Dùng stdio cho một client trên máy cá nhân.

## Xử lý lỗi

- **Missing TANCA_TOKEN** — kiểm tra đường dẫn sau `--env` và tên biến trong tệp.
- **401** — đăng nhập lại vào Tanca, lấy header từ yêu cầu mới và khởi động lại MCP sau khi cập nhật token. Kiểm tra cả biến môi trường cũ nếu lỗi vẫn còn.
- **403** — kiểm tra tài khoản có thể thực hiện cùng thao tác trên giao diện Tanca và đang dùng đúng shop, chi nhánh.
- **Sai ID hoặc ngày** — lấy ID từ kết quả tra cứu, dùng `YYYY-MM-DD` cho ngày và `YYYY-MM-DD HH:mm:ss` cho ngày giờ. Đặt giá trị có khoảng trắng trong dấu nháy.
- **Lỗi sau một thao tác ghi** — tra lại nhật ký hoặc danh sách đề xuất trước khi gửi lại để tránh ghi trùng.

## Kiểm tra khi phát triển

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Mã gọi API nằm trong `src/services/`, các tool MCP trong `src/tools/`, phần định dạng kết quả dùng chung trong `src/format.rs`, server MCP trong `src/main.rs` và CLI trong `src/bin/tanca-cli.rs`. Kiểm thử với API giả lập nằm trong `tests/`.
