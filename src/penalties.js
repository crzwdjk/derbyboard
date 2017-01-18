"use strict";

function relpath(moveup, path) {
    var common = document.location.protocol + '//' + document.location.host;
    var parts = document.location.pathname.split('/').slice(0, -moveup);
    return common + '/' + parts.join('/') + path;
}

function loadRows(team, callback) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', relpath(1, "gameroster/" + team));
    xhr.onreadystatechange = function () {
        if(xhr.readyState === XMLHttpRequest.DONE && xhr.status === 200) {
            callback(JSON.parse(xhr.responseText), team);
        }
    };
    xhr.send();
}
var penaltycodes = [
    'A', 'B', 'C', 'E', 'F', 'G', 'H', 'I', 'L', 'M', 'N', 'O', 'P', 'S', 'U', 'X', 'Z'
];

function keyhandler(e) {
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    switch (e.key) {
    case "ArrowUp": case "Up":
        if (this.previousElementSibling) {
            this.previousElementSibling.previousElementSibling.focus();
        }
        break;
    case "ArrowDown": case "Down":
        if (this.nextElementSibling.nextElementSibling) {
            this.nextElementSibling.nextElementSibling.focus();
        }
        break;
    case "ArrowRight": case "Right":
        var row = document.querySelector('#team2penalties tbody tr:nth-child(1)');
        row.focus();
        break;
    case "ArrowLeft": case "Left":
        var row = document.querySelector('#team1penalties tbody tr:nth-child(1)');
        row.focus();
        break;
    // TODO: backspace, delete to remove penalty
    // TODO: minus to move penalty to prev jam, plus to next jam
    default:
        if (penaltycodes.indexOf(e.key.toUpperCase()) != -1) {
            var code = e.key.toUpperCase();
            var tableid = this.parentElement.parentElement.id;
            var team = /team(\d)penalties/.exec(tableid)[1];
            var skater = this.querySelector('td.rosternum').innerText;
            penalty(team, skater, code);
        }
    }
}

function injectRows(table, skaters) {
    var body = table.tBodies[0];
    var template = document.getElementById('penaltyrow').content;
    var i;
    for (i = 0; i < skaters.length; i++) {
        var skaternum = template.querySelectorAll('.rosternum')[0];
        skaternum.innerText = skaters[i].number;
        var row = document.importNode(template, true);
        body.appendChild(row);
        var toprow = body.children[body.childElementCount - 2];
        var bottomrow = body.children[body.childElementCount - 1];
        toprow.addEventListener('click', function (e) {
            this.focus()
        });
        bottomrow.addEventListener('click', function (e) {
            this.previousElementSibling.focus();
        });
        toprow.addEventListener('keydown', keyhandler);
    }
}

function load() {
    var t = -1;
    for (t = 1; t <= 2; t++) {
        loadRows(t, function (team, tn) {
            var table = document.getElementById("team"+tn+"penalties");
            var name = document.getElementById("team"+tn+"name");
            injectRows(table, team.skaters);
            name.innerText = team.name;
        });
    }
}

function update_board(team, penalties) {
    var tbody = document.getElementById("team"+team+"penalties").tBodies[0];
    var n = tbody.childElementCount;
    var row;
    for (row = 0; row < n; row += 2) {
        var toprow = tbody.children[row];
        var bottomrow = tbody.children[row + 1];
        var skaternum = toprow.children[0].innerText;
        var penaltycells = toprow.getElementsByClassName('penalty');
        var jamcells = bottomrow.getElementsByClassName('penalty');
        var i;
        var skaterpenalties = penalties[skaternum];
        for (i = 0; i < skaterpenalties.length; i++) {
            var p = skaterpenalties[i];
            penaltycells[i].innerText = p.code;
            jamcells[i].innerText = p.jam;
        }
    }
}

function penalty(team, skater, penaltycode) {
    var xhr = new XMLHttpRequest();
    xhr.open('POST', document.location + '/' + team);
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onreadystatechange = function () {
        if (xhr.readyState == XMLHttpRequest.DONE && xhr.status == 200) {
            var resp = JSON.parse(xhr.responseText);
            update_board(team, resp);
        }
    };
    xhr.send(JSON.stringify({skater: skater, code: penaltycode}));
}

function update(team) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', document.location + team);
    xhr.onreadystatechange = function () {
        if (xhr.readyState == XMLHttpRequest.DONE && xhr.status == 200) {
            var resp = JSON.parse(xhr.responseText);
            update_board(resp);
        }
    }
    xhr.send();
}

load();

/* UI concept: highlighted row.
   Arrow keys up and down move row up and down
   Arrow keys left and right move between teams.

   Pushing a penalty code records that penalty (via XHR).
   +/- to adjust jam #.
   and XHR response includes new penalty state.

   Also a periodic (1 sec?) poll to update penalties
*/
