## Language Detection

### Input:
- `input_text`: String (optional)
- `storage_type`: String (optional)
- `input_path`: String (optional)
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
Identifies the language of a given text input either through `input_text` or a text file from one of the provided storage types. 

---

### Example JSON Payload:
```json
{
    "storage_type": "memory",
    "input_path": "files:spanish_text_1kb"
}