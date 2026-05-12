import requests

payload = {
    "file_path": "/shared/uploads/sample_contract_acme.txt",
    "doc_id": "f091c796-54a9-4e9d-824e-e4046254c757",
    "kb_id": "fase6_test_kb_1778353794",
    "mime_type": "text/plain"
}

try:
    resp = requests.post("http://localhost:8091/parse", json=payload)
    print(resp.status_code)
    print(resp.text)
except Exception as e:
    print(f"Request failed: {e}")
