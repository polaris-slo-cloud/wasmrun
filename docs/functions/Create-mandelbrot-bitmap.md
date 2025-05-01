## Create Mandelbrot Bitmap

### Input:
- `image_size`: usize
- `storage_type`: String
- `output_path`: String
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function generates a Mandelbrot fractal image in the PBM (portable bitmap) format. The resulting bitmap can be stored in local storage, S3, or redis in-memory.

---

### Example JSON Payload:
```json
{
  "image_size": 512,
  "storage_type": "local",
  "output_path": "/images/mandelbrot.pbm"
}
