[Phase 2] Add Prometheus Metrics and Monitoring

**Feature Category**
- [ ] New Provider Support
- [ ] Streaming Support
- [ ] Tool Calls
- [x] Performance
- [ ] Other

**Description**
Add Prometheus metrics endpoint for monitoring proxy health, request latency, provider status, and usage statistics.

**Acceptance Criteria**
- [ ] Add `/metrics` endpoint for Prometheus scraping
- [ ] Track request latency (histogram)
- [ ] Track request count by provider/model (counter)
- [ ] Track active sessions (gauge)
- [ ] Track provider availability (gauge)
- [ ] Track error rates (counter)
- [ ] Add optional Grafana dashboard JSON
- [ ] Documentation for monitoring setup

**Technical Notes**

Metrics to expose:
```
# Request latency
locai_proxy_request_duration_seconds{provider="opencode",model="claude-3.5-sonnet"}

# Request count
locai_proxy_requests_total{provider="opencode",status="success"}

# Active sessions
locai_proxy_active_sessions

# Provider health
locai_proxy_provider_health{provider="opencode"}
```

Implementation:
- Use `prometheus` crate
- Use `axum-prometheus` middleware
- Configure scrape interval

**Dependencies**
- None

**Priority**
- [ ] Must Have
- [ ] Should Have
- [x] Nice to Have (production deployments)

**Estimated Effort**
2-3 days
