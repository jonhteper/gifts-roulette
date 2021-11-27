# gifts-roulette
A cli tool for gift exchange

## Use
```
gifts-roulette [OPTIONS] <COMMAND>
```
### Shuffle participants and send emails
```
gifts-roulette start
```
#### Set the excel input file
```
gifts-roulette -i <INPUT-FILE> start
```
Default value: input.xlsx

#### Set the output json file to save the couples
```
gifts-roulette -o <OUTPUT-FILE> start
```
Default value: db.json