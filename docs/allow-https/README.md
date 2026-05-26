# Remote Deck — HTTPS và giữ màn hình sáng

Hướng dẫn dùng **link HTTPS** trên điện thoại để tính năng **giữ màn hình sáng** (Wake Lock API) hoạt động, và cách **chấp nhận chứng chỉ** tự ký lần đầu.

## Giao diện Remote trên điện thoại

Mở link HTTPS trên Safari/Chrome cùng Wi‑Fi với PC. Bạn sẽ thấy grid app (Stream Deck) và hai nút góc phải:

- **Toàn màn hình** — ẩn thanh địa chỉ, dùng như app treo tường
- **Giữ màn hình sáng** (biểu tượng mặt trời) — không cho màn hình tự tắt khi treo điện thoại làm bảng điều khiển

![TrayLink Remote trên điện thoại — grid app và nút giữ màn hình sáng](../mobile%20screen%20shot.jpg)

## Vì sao cần HTTPS?

Trình duyệt chỉ cho phép **Screen Wake Lock API** (`navigator.wakeLock`) trong **secure context**:

| Cách mở | Wake Lock (giữ màn hình sáng) |
|---------|-------------------------------|
| `https://192.168.x.x:8766/remote` | Có |
| `http://192.168.x.x:8765/remote` | Không |

- **HTTP** qua IP LAN (`http://192.168.1.x:8765`) **không** được coi là secure context → Wake Lock bị chặn → màn hình vẫn tự tắt dù đã bật nút mặt trời.
- **HTTPS** (`https://192.168.x.x:8766`) → Wake Lock hoạt động bình thường.

TrayLink tự chạy thêm server HTTPS trên **port = port HTTP + 1** (mặc định HTTP `8765` → HTTPS `8766`), dùng **chứng chỉ tự ký** cho IP LAN của máy bạn. Chỉ dùng trong mạng nhà/văn phòng — không phải chứng chỉ công cộng.

> **Lưu ý:** Wake Lock chống **tắt màn hình theo thời gian**, không chống **khóa màn hình bằng nút nguồn**. Tắt **Tiết kiệm pin** / **Low Power Mode** trên iPhone nếu màn hình vẫn tắt nhanh.

## Lấy link HTTPS

1. Mở **TrayLink** trên PC → **Open Dashboard**
2. Tab **Overview** → mục **Remote Deck — HTTPS (giữ màn hình sáng)**
3. Copy link dạng `https://192.168.1.x:8766/remote` hoặc quét **QR HTTPS**
4. Mở link đó trên điện thoại (cùng Wi‑Fi với PC)

Nếu bật API token: thêm `?token=...` vào URL (Dashboard copy sẵn).

## Chấp nhận chứng chỉ tự ký (lần đầu)

Lần đầu mở HTTPS, trình duyệt báo kết nối không riêng tư / không tin cậy — **bình thường** vì chứng chỉ do TrayLink tự tạo, không qua CA công cộng.

<table>
  <tr>
    <td width="50%" valign="top">
      <p><strong>Bước 1 — Cảnh báo</strong></p>
      <p>Safari hoặc Chrome báo kết nối không riêng tư — bình thường, chọn tiếp tục.</p>
      <img src="alert%20https.jpg" alt="Cảnh báo chứng chỉ HTTPS — kết nối không riêng tư" width="280" />
    </td>
    <td width="50%" valign="top">
      <p><strong>Bước 2 — Cho phép</strong></p>
      <p>Chọn <strong>Tiếp tục</strong>, <strong>Advanced</strong> → <strong>Proceed</strong>, hoặc <strong>Visit this website</strong> (tùy trình duyệt).</p>
      <img src="allow%20https.jpg" alt="Cho phép truy cập HTTPS với chứng chỉ tự ký" width="280" />
    </td>
  </tr>
</table>

Sau khi chấp nhận một lần, trình duyệt thường nhớ cho cùng IP/port trên thiết bị đó.

## Bật giữ màn hình sáng

1. Mở Remote bằng link **HTTPS** (không dùng link HTTP nếu cần giữ sáng)
2. Chạm nút **mặt trời** góc phải — nút sáng xanh = đang bật
3. (Khuyến nghị) Chạm nút **toàn màn hình** bên cạnh
4. Nếu vẫn tắt: chạm nhẹ màn hình để gia hạn Wake Lock; tắt Tiết kiệm pin

Nếu bạn mở bằng HTTP mà bật giữ sáng, app có thể **tự chuyển** sang HTTPS (nếu server HTTPS đang chạy).

## HTTP vs HTTPS — tóm tắt

| | HTTP `:8765` | HTTPS `:8766` |
|---|-------------|---------------|
| Mở app trên PC từ điện thoại | Có | Có |
| Giữ màn hình sáng (Wake Lock) | Không | Có |
| Cần chấp nhận chứng chỉ | Không | Có (lần đầu) |

**Khuyến nghị:** Điện thoại treo làm bảng điều khiển → luôn dùng link **HTTPS** trong Overview.

## Xử lý sự cố

| Triệu chứng | Gợi ý |
|-------------|--------|
| Không mở được `https://...:8766` | Kiểm tra PC và điện thoại cùng Wi‑Fi; Restart Server trong Dashboard; firewall cho phép port 8766 |
| Vẫn báo chứng chỉ sau khi Allow | Xóa cache trang / thử tab ẩn danh rồi Allow lại |
| Nút mặt trời bật nhưng màn hình vẫn tắt | Đảm bảo URL bắt đầu bằng `https://`; tắt Tiết kiệm pin; bật toàn màn hình |
| Chỉ cần mở app, không cần giữ sáng | Có thể dùng link HTTP `:8765` |

## Liên quan

- [README chính](../../README.md) — Remote Deck, API
- Port mặc định: HTTP `8765`, HTTPS `8766` (HTTPS = HTTP + 1)
