## Audio Generation

### Input:
- `audio_size`: usize
- `storage_type`: String
- `path`: String
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function generates a sine wave-based audio file (in WAV format) with a target file size specified by `audio_size` in kb. The generated audio is then saved based on the `storage_type` and `path`.

---

### Example JSON Payload:
```json
{
  "audio_size": 1024,
  "storage_type": "redis",
  "path": "sin-wave.wav",
}