## Image Resize

### Input:
- `scale_factor`: f64
- `storage_type`: String
- `path`: String
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function resizes an image using a specified `scale_factor`, which adjusts the dimensions proportionally (e.g., `2.0` doubles the size, `0.5` halves it). A default test image (720x480 pixels, red) is used for resizing, and the resized image is saved in JPEG format. The storage location is determined by `storage_type` and `path`.

---

### Example JSON Payload:
```json
{
  "scale_factor": 2.0,
  "storage_type": "redis",
  "path": "test_image.jpg",
}