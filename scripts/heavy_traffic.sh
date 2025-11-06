#!/bin/bash
wrk -t8 -c100 -d30s -s post.lua http://localhost:5000/logs

