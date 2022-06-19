cleaning_managment_rs is a cli software used to generate a table containing given names spread and rotated over a set period of time while skipping Hebrew public holidays.
After a table has been created cleaning_managment_rs can send SMS reminders to the person sloted on the same day in a given time.

Create a table by executing the programn in the Command Line/Terminal. 

Note: If configuration files have not been created example files would be generated automatically in your Documents folder under a "cleaning_managment" folder.
The config files contains: "config.json" - the main configuration file, "names.csv" - the file listing the names of all people to be added to the table, "excluded_hebcal" - file used to excluded holidays so the program won't skip over them (i.e. "Yom Yerushalayim" should not be skipped since no holiday is usually given that day.).
Fill the configuration files as you see fit and run the program again.

After running the executable the created table should be in the "output" folder under "cleaning_managment".

Once a table is created you can run the software with the -r flag to start the sending process.
While in sending mode you can type "help" to list avilable commands to execute.
