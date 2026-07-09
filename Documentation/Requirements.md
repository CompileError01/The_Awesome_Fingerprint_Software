### Functional Requirements

- Automatic synchronization of users, fingerprints, and permissions across all door controllers.
- Doors must continue to function even if the network connection fails (offline capability).

- Support multiple user roles (e.g., **Normal User** and **Group Leader**) with different access rights.

- Group Leaders can enable **Workshop Mode**, keeping a door unlocked via a button.

- Configurable maximum unlock duration, either per user or per user group.

- Support at least **two fingerprints per user**.

- User access permissions should have an optional expiration date.

- Maintain a searchable audit log of configuration changes (who created users or changed permissions and when).

- Optionally maintain a searchable access log (admin-only) showing who entered which door and when.

- Web interface should be accessible from any device (e.g., smartphone), not just the Raspberry Pi.

- Simple and well-documented procedure for adding or removing doors from the system.