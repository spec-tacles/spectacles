# Gateway

Gateway outputs Discord gateway data to STDOUT in BSON format.

## Config

```toml
token = ""

[gateway]
intents = 0

# everything below is optional

[gateway]
events = []

# disjoint from type = "range"
[shards]
type = "bucket"
bucket_id = 0
concurrency = 0
total = 0

# disjoint from type = "bucket"
[shards]
type = "range"
from = 0
to = 0
total = 0

[api]
version = 0
base = { url = "", use_http = true }
timeout = ""
```
