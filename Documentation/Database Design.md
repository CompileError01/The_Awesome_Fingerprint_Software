## Basic Requirements:

| Table               | Purpose                                             | Key Fields                                                               |
| ------------------- | --------------------------------------------------- | ------------------------------------------------------------------------ |
| **Door**            | Stores registered doors                             | `doorID`, name, IP                                                       |
| **User**            | Stores user information                             | `userID`, first name, last name, email, password hash, role, valid until |
| **Hands**           | Links users to one or more fingerprints             | `fingerID`, `userID`                                                     |
| **Finger**          | Stores fingerprint templates in a separate database | `fingerID`, fingerprint file                                             |
| **RFID (optional)** | Links RFID cards to users                           | `userID`, RFID code                                                      |
| **Access**          | Defines door permissions and unlock duration        | `userID`, `doorID`, open time                                            |
| **Auth Log**        | Records door access events                          | time, `userID`, `doorID`, `fingerID`, event                              |
| **Conf Log**        | Records configuration changes                       | time, `userID`, change details                                           |


# Database layout:
```
/Database
	
	/Users
		- LinusTorvalds.json
		- EdwardSnowden.json
	
	/Doors
		- Door01.json
		- Door02.json
	
	/Logs
		- Auth.log
		- Conf.log
```
