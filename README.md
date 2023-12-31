# Arke command line chatting application

## Getting started (Local test)
### Setup ganache
1. Clone the project
2. Install ganache from [https://trufflesuite.com/ganache/](https://trufflesuite.com/ganache/)
3. Start new workspace in ganache 
   ![setup ganache 3](screenshots/setup_ganache_3.png)
4. Add project with the path arke_application\store\truffle-config.js
5. Set the server with <br/>
    HOSTNAME: 127.0.0.1 <br/>
    PORT NUMBER: 9545 <br/>
    NETWORK ID: 5777 <br/>
    ![setup ganache 5](screenshots/setup_ganache_5.png)
6. Install truffle from [https://trufflesuite.com/docs/truffle/how-to/install/](https://trufflesuite.com/docs/truffle/how-to/install/)
7. In a command prompt, Navigate to store folder and deploy the Ethereum contract of key-value store with the commands below:
   ```sh
   truffle develop
   compile
   migrate
   ```
   Ganache should show that the contract is deployed now.
8. Keep ganache running

### Setup for testing the application
1. Navigate to the client folder, update the contract address on the file private_chat_and_pay.rs to the deployed contract address as shown on ganache. Also update the contract address on the file delete_friend.rs to the same deployed contract address.
   ![setup for testing the application 1](screenshots/setup_for_testing_the_application_1.png)
2. Duplicate the client folder to simulate two users.
3. In command prompt 1, Navigate to client folder and start client for user1 with the command below:
   ```sh
   cargo run --release
   ```
4. In command prompt 2, Navigate to the copied client folder and start client for user2 with the command below:
   ```sh
   cargo run --release
   ```
5. In command prompt 3, Navigate to database_server folder and start application server with the command below:
   ```sh
   cargo run --release
   ```
6. In command prompt 4, Navigate to server folder and start key-issuing authority with the command below:
   ```sh
   cargo run --release
   ```
7. In command prompt 5, Navigate to server folder and start registration authority with the command below:
   ```sh
   cargo run --release
   ```

### Testing the application
![testing the application](screenshots/testing_the_application.png)
1. In the first runs for both clients, select "My Info" from the main menu to fill the personal information of the users. Give different IDs and eth wallet addresses for two users. Eth addresses can be selected from the account address provided in the previously setup ganache workspace.
2. For user1, select "Contact Discovery" from the main menu to add user2 to the friend list. For user2, select "Contact Discovery" from the main menu to user1 to the friend list. Contact discovery takes a long time even in release mode due to client deserializing setup details such as pp_zk from the server. In real-world implementation, those public parameters are pre-downloaded to the client during the installation of the application.
3. After contact discovery finished for both users, for each user, select "Contacts" from the main menu to verify that the discovered user is added to the contact book.
4. Both users can select each other in their own friend list to start private chat or pay.