# Zenoh Swiss Army Knife
This is a command line tool, simply called `zenoh` that allows to do easily experiment with Zenoh's publication, 
subscriptions, query, and queriables.

## Subscribing
Creating a subscriber is extremely easy, as shown below:

     zenoh sub zenoh/greeting

You can also use key expressions, as in:

    zenoh sub zenoh/*

## Publishing
Making publications is extremely staight forward, below are some examples.

To make a single publication, you can do 

     zenoh pub zenoh/greeting hello

To publish multiple messages you can specify the `--count` option and use the {N} macro
if you want to diplay the cardinal number of the message:

    zenoh pub --count 10  zenoh/greeting "This is the {N}th time I am saying hello!"

You can also publish messages periodically, by providing a duration in milliseconds:

    zenoh pub --count 10 --period 1000 zenoh/greeting "This is the {N}th time I am saying hello -- every second!"

Sometimes it is handy to publish data using files, that can be easily achieved by enabling file-based input, 
as shown below:

    zenoh pub -file --count 10 --period 1000 zenoh/greeting /path/to/myfile


