# Arke command line chatting application

## Getting started
### Setup ganache
1. Clone the project
2. Install ganache at [https://trufflesuite.com/ganache/](https://trufflesuite.com/ganache/)
3. Start new workspace in ganache 
4. Add project with the path arke_application\store\truffle-config.js
5. Set the server with 
    HOSTNAME: 127.0.0.1
    PORT NUMBER: 9545
    NETWORK ID: 5777
6. Keep ganache running

### Setup for testing the application
1. Duplicate the client folder to simulate two users.
2. In a command prompt, Navigate to client folder and start client for user1.
   ```sh
   cargo run --release
   ```
3. In another command prompt, Navigate to the copied client folder and start client for user2.
   ```sh
   cargo run --release
   ```
4. In another command prompt, Navigate to server folder and start server.
   ```sh
   cargo run --release
   ```

### Testing the application
1. In the first runs for both clients, select "My Info" from the main menu to fill the personal information of the users. Give different nicknames and eth addresses for two users. Eth addresses can be selected from the account address provided in the previously setup ganache workspace.
2. Select "Contact Discovery" from the main menu for each users to add themselves to the contact book of each other. Contact discovery takes a long time due to deserializing setup details such as pp_zk from the server.
3. After contact discovery finished for both users, select "Contacts" to verify that the discovered user is added to the contact book.
4. Both users select each other in the contact book to start chatting.
5. Type in any message to chat.


