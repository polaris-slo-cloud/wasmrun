## Decrypt Message

### Input:
- `key`: String (must be exactly 32 characters long)  
- `encrypted_data`: String (optional, Base64-encoded)  
- `storage_type`: String (optional)  
- `input_path`: String (optional)  
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function decrypts an encrypted message using the AES-256-GCM decryption algorithm. The decryption key is required and must be exactly 32 characters long. The encrypted message can be provided directly by `encrypted_data` or retrieved from storage, depending on the `storage_type`.

---

### Example JSON Payload:
```json
{
  "key": "12345678901234567890123456789012",
  "encrypted_data": "BASE64_ENCODED_ENCRYPTED_DATA"
}
