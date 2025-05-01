## Fuzzy Search

### Input:
- `search_term`: String  
- `storage_type`: String (optional)  
- `input_path`: String (optional)  
- `bucket`: String (optional, required only if storage_type is s3)  

### Description:
This function performs a fuzzy search to find approximate matches for a given `search_term` within a text-based data source. The data source can be loaded from different storage types such as local files, S3, or redis in-memory storage.

---

### Example JSON Payload:
```json
{
  "search_term": "apple",
  "storage_type": "redis",
  "input_path": "search_text_1kb.txt"
}
