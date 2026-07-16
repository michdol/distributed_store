1 leader, n followers

Leader must detect followers. - follower contacts leader?

Leader pushes WAL through socket - sync vs. async (strong vs. weak consistency)

Followers accept reads only.

When leader dies, follower should wake up


```
curl -X POST http://127.0.0.1:3000/set -H "Content-Type: application/json" -d '{"key": "user_name", "value": "Alice"}'
```
