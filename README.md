# CryptoWrap

CryptoWrap is a payment gateway/processor architected for secure, reliable, and seamless cryptocurrency transactions.
Providing a unified interface for both inbound and outbound payments. The project prioritizes simplicity and ease of integration, offering a lightweight and fast solution that operates as an extensible wrapper (API layer) for various blockchains.

### Coins
- Monero 
- Litecoin

#### Features
- HTML page for accepting crypto payments in 3 different modes:
1. WebSocket (fastest, requires .js, not implemented yet)
2. HTTPS polling (reliable, requires .js, default, implemented)
3. No-JavaScript (requires refresh html tag and backend logic, not implemented yet)

You can use your own customised HTML template using same API endpoints.

- Accept, store and send coins via isolated `virtual` accounts. <br>
For systems with multiple users, where funds must be safely separated and managed.


## Technology Stack

This project is built using a robust and modern technology stack, orchestrated within Docker containers for easy deployment and scalability:

- **Reverse Proxy/Load Balancer:** Nginx (default) / Nginx + HAProxy
- **Backend:** Rust (Axum framework)
- **API Documentation:** Swagger UI and OpenAPI.json
- **Database Interaction & Migrations:** Sea-ORM
- **Database:** PostgreSQL

## License

This project is open-source and licensed under the Affero General Public License (AGPL), promoting freedom and encouraging the sharing of modifications or extended versions, even when used as a web service.
