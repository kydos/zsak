# Zenoh Swiss Army Knife
This is a command line tool, simply called `zenoh` that allows to do easily experiment with Zenoh's publication, 
subscriptions, query, and queriables.

## Scouting
The scouting command allow to scout for Zenoh runtimes using the configuration provided if any, or the 
default otherwise. You need to provide a scouting interval that indicates for how long zenoh will be actively
looking for other nodes. Below an example:

    zenoh scout 2


## Subscribing
Creating a subscriber is extremely easy, as shown below:

     zenoh subscribe zenoh/greeting

You can also use key expressions, as in:

    zenoh subscribe zenoh/*

## Publishing
Making publications is extremely staight forward, below are some examples.

To make a single publication, you can do 

     zenoh publish zenoh/greeting hello

To publish multiple messages you can specify the `--count` option and use the {N} macro
if you want to diplay the cardinal number of the message:

    zenoh publish --count 10  zenoh/greeting "This is the {N}th time I am saying hello!"

You can also publish messages periodically, by providing a duration in milliseconds:

    zenoh publish --count 10 --period 1000 zenoh/greeting "This is the {N}th time I am saying hello -- every second!"

Sometimes it is handy to publish data using files, that can be easily achieved by enabling file-based input, 
as shown below:

    zenoh publish -file --count 10 --period 1000 zenoh/greeting /path/to/myfile


