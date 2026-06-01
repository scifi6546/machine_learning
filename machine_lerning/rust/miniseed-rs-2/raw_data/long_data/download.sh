#! /bin/bash
wget  -O "./data.mseed" "https://service.earthscope.org/fdsnws/dataselect/1/query?net=AK&sta=GHO&cha=BHN&starttime=2016-06-10T06:30:00.000&endtime=2016-06-13T06:30:00.000&format=miniseed"