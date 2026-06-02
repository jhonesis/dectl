# System Prompt — Microservice Project
> **Instructions for the model**: You're working on a microservices architecture.

---

## Microservice-Specific Guidelines

**Service Design**:
- Single responsibility per service
- Independent deployability
- Stateless where possible

**Communication**:
- Document inter-service contracts
- Handle failures gracefully (circuit breakers, retries)
- Use appropriate protocols (HTTP, gRPC, messages)

**Infrastructure**:
- Configuration via environment variables
- Health checks for all services
- Logging aggregation

**Documentation**:
- Keep .dec/knowledge/service-registry.md updated
- Document Docker setup in .dec/docker/
