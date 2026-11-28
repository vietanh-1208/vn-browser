# 🌐 VN Surf

Trình duyệt web siêu nhẹ, bảo mật và tốc độ cao dành cho **Windows / Linux / macOS** (Rust + WebView) và **Android** (Kotlin + WebView).

## ✨ Tính năng nổi bật
- **🛡️ Chặn quảng cáo & chống nhảy tab tự động (VN Shield)**: Chặn banner quảng cáo, quảng cáo YouTube, link cá độ/pop-under và tự động chặn tab quảng cáo bật lên trái phép.
- **🔐 Quản lý mật khẩu bảo mật cao**:
  - Mã hoá cục bộ bằng **Windows DPAPI** (PC) và **Android Keystore (`AES256_GCM`)** (Android).
  - Bắt buộc xác minh bằng **mật khẩu / PIN / vân tay của máy** (`Windows Hello` / `BiometricPrompt`) mỗi khi mở xem, sao chép hoặc tự điền tài khoản.
  - Tự động gợi ý lưu mật khẩu khi đăng nhập và nút **`🔑 Điền TK`** nhanh trên thanh địa chỉ.
- **🕵️ Tab Ẩn danh & Khoá PIN**: Hỗ trợ tab ẩn danh riêng biệt; bản PC hỗ trợ khoá tab ẩn danh bằng mã PIN (đổi/gỡ PIN bắt buộc xác minh mật khẩu máy).
- **🌐 Dịch trang nhanh đa ngôn ngữ**: Nút **`🌐 Dịch`** ngay trên thanh địa chỉ hỗ trợ dịch trang sang Tiếng Việt, Anh, Nhật, Hàn, Trung, Pháp, Đức, Tây Ban Nha, Thái, Nga.
- **⚡ Tối ưu RAM & Giao diện hiện đại**:
  - Tự động ngủ tab nền tiết kiệm RAM (PC), chế độ đọc sách, Picture-in-Picture (PiP), nền tối/sáng kiểu Chrome.
  - Vượt chặn DNS nhà mạng với **DoH (1.1.1.1)** & hỗ trợ Proxy tuỳ chỉnh (PC).
  - Tự động kiểm tra & cập nhật trực tiếp từ **GitHub Releases** (PC).

## 📥 Tải về & Cài đặt
Vào mục **[Releases](https://github.com/vietanh-1208/vn-browser/releases)** để tải bản mới nhất:
- **Windows:** `VN-Surf-Setup-v1.1.0.exe` (bộ cài đặt tự động tạo lối tắt Desktop & Start Menu)
- **Android:** `VN-Surf-Android-v1.1.0.apk` (cài trực tiếp trên Android 8.0+)
- **Linux / macOS:** Tải gói tương ứng trong Releases.

## 🛠️ Tự biên dịch từ mã nguồn
### PC (Rust)
```bash
cargo test
cargo build --release
```

### Android (Gradle)
```bash
cd android
gradle assembleRelease
```
