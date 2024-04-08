# Numov 0.1.2
#### A simple CLI program which helps organize and manage libraries of matroska files!

In order to use this program, files must be organized in the following structure:
```
root_folder
    |___ movie_title (year) 
    |       |___ movie.mkv
    |
    |___ movie_title (year)
            |___ movie.mkv
```
Each subdirectory must follow this naming convention `movie title here (year)` in order to successfully extract title/year information **unless** that format is written into the file's metadata.

### Usage
- `-P <path>` initializes and updates the database
- `-C, --csv` outputs contents of database into csv file in cwd
- `-R, --rename` bulk renames parent folders in a standard, readable fashion
    - Will rename files within directory provided with `-P <path>`
- `-d, --dataframe` outputs condensed dataframes of requested info
     - possible values: [`subs`, `audio`, `channels`, `year`, `all`]
- `--reset` will remove existing numov database

#### Letterboxd functionality
- `-L, --letterboxd <LB username>` users can map the ratings of any **non-private** letterboxd user.

#### Example command:
`numov -P path/to/root -L deathproof --csv --rename`

### Optional: MkvPropEdit Dependency
If a user has [mkvpropedit](https://mkvtoolnix.download/doc/mkvpropedit.html) in their path, the files `title` metadata will be overwritten for reliable future data retrieval. Numov does not make any writes to any user files in any other way. Numov will operate fine if mkvpropedit is not callable. 

Tested on Windows 11 and Arch Linux

### Other
1. Numov does not collect any user data. 
2. Outside of the mkvpropedit write, Numov will not write to any existing files.
3. As of version 0.1.1, Numov can only manage a single root directory at a time. Multiple root management is a possible future implementation. 
