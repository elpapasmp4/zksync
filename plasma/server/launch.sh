#!/bin/sh

ps auxw | grep server | grep -v grep > /dev/null

if [ $? != 0 ]
then
    ./run.sh 2>&1 | tee log.log
fi