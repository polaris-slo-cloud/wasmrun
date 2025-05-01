## Encrypt Message

### Input:
- `key`: String (must be exactly 32 characters long)  
- `message`: String (optional)  
- `storage_type`: String (optional)  
- `input_path`: String (optional)  
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function encrypts a given message using the AES-256-GCM encryption algorithm. The key is a required input and must be exactly 32 characters long. The message can be provided directly by `message` or retrieved from storage, depending on the `storage_type`.

---

### Example JSON Payload:
```json
{
  "key": "12345678901234567890123456789012",
  "message": "Hello, World!"
}
