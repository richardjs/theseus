import argparse
import os
import sys
import time
from datetime import datetime
from subprocess import Popen, PIPE

import flask
from flask import request

from .config import API_ENGINE_PATH, API_PORT

app = flask.Flask(__name__)


@app.route('/theseus')
def think():
    gameid = request.args.get('id')
    num_players = request.args.get('players')
    pawn1 = request.args.get('pawn1')
    pawn2 = request.args.get('pawn2')
    walls1 = request.args.get('wallcount1')
    walls2 = request.args.get('wallcount2')
    wall_centers = request.args.get('wallcenters')
    turn = request.args.get('turn')

    tqbn = pawn1.zfill(2) + pawn2.zfill(2) + \
        walls1.zfill(2) + walls2.zfill(2) + wall_centers + turn

    engine = Popen((
        API_ENGINE_PATH,
        tqbn,
    ), stdout=PIPE, stderr=PIPE)

    move = engine.stdout.readline().strip().decode('utf-8')
    rawlog = engine.stderr.read().strip().decode('utf-8')
    log = '> ' + rawlog.replace('\n', '\\n> ')

    r = flask.Response('{"move": "%s", "log": "%s"}' % (move, log))
    r.headers['Content-Type'] = 'application/json'
    r.headers['Access-Control-Allow-Origin'] = '*'
    return r


app.run(debug=False, port=API_PORT)