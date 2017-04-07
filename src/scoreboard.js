/* Expects a score update like: 
{ score: [ team1score, team2score],
  jamscore: [ team1score, team2score] },
*/

function setInner(id, content) {
    document.getElementById(id).innerText = content;
}

function format_time(time) {
    var mins = (time / 60)|0;
    var secs = (time % 60);
    var secstr = secs < 10 ? ("0" + secs) : ("" + secs);
    if (mins > 0) {
        return mins + ":" + secstr;
    } else {
        return secstr;
    }
}

function updateClock(data) {
    var label = 'Jam 1';
    var clock = '2:00';
    for (var l in data) {
        switch (l) {
        case 'timeout':
            label = 'Official Timeout';
            clock = data.timeout;
            break;
        case 'team_timeout':
            label = 'Team Timeout';
            clock = data.team_timeout[1];
            break;
        case 'review':
            label = 'Official Review';
            clock = data.review[1];
            break;
        case 'jam':
            label = 'Jam ' + data.jam[0];
            clock = data.jam[1];
            break;
        case 'lineup':
            label = 'Lineup';
            clock = data.lineup;
            break;
        case 'intermission':
            label = 'Halftime';
            clock = data.intermission;
            break;
        case 'time_to_derby':
            label = "Time To Derby";
            clock = data.time_to_derby;
            break;
        }
    }
    setInner('aclabel', label);
    setInner('actime', format_time(clock.secs));
}

function updateScore(data) {
    setInner('totalscore1', data.score[0]);
    setInner('totalscore2', data.score[1]);
    setInner('jamscore1', data.jamscore[0]);
    setInner('jamscore2', data.jamscore[1]);
    setInner('periodtime', format_time(data.gameclock[1].secs));
    setInner('ptlabel', "Period " + data.gameclock[0]);
    updateClock(data.activeclock);
    setTimeouts(data.timeouts, data.reviews, data.activeclock);
}
function setTeamDots(teambox, timeouts, reviews, in_timeout, in_review) {
    var todots = teambox.getElementsByClassName('tobox')[0].children;
    for (let i = 0; i < todots.length; i++) {
        if (i < timeouts) {
            todots[i].className = "dot";
        } else if (i == timeouts && in_timeout) {
            todots[i].className = "dot blinking";
        } else {
            todots[i].className = "dot hidden";
        }
    }
    var ordot = teambox.getElementsByClassName('orbox')[0].children[0];
    if (in_review) {
        ordot.className = "dot blinking";
    } else if (reviews > 0) {
        ordot.className = "dot";
    } else {
        ordot.className = "dot hidden";
    }
}

function setTimeouts(timeouts, reviews, activeclock) {
    // TODO: only do this if TO state changes
    console.log(activeclock);
    let toTeam = 0;
    if ('team_timeout' in activeclock) {
        toTeam = activeclock.team_timeout[0] == 'Home' ? 1 : 2;
    }
    let orTeam = 0;
    if ('review' in activeclock) {
        orTeam = activeclock.team_timeout[0] == 'Home' ? 1 : 2;
    }
    for (let team = 1; team <= 2; team++) {
        var teambox = document.getElementById('team' + team);
        setTeamDots(teambox, timeouts[team - 1], reviews[team - 1],
                    toTeam == team, orTeam == team);
    }
}

function updater() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', document.location + "/update");
    xhr.onreadystatechange = function () {
        if(xhr.readyState === XMLHttpRequest.DONE && xhr.status === 200) {
            updateScore(JSON.parse(xhr.responseText));
        }
    };
    xhr.send();
}

function send_command(update) {
    var xhr = new XMLHttpRequest();
    xhr.open('POST', document.location + "/update");
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onreadystatechange = function () {
        if(xhr.readyState === XMLHttpRequest.DONE && xhr.status !== 200) {
            
        }
    };
    xhr.send(JSON.stringify(update));
}    
    

function clickhandler(e) {
    var el = document.body;
    var rfs = 
           el.requestFullScreen
        || el.webkitRequestFullScreen
        || el.mozRequestFullScreen
        || el.msRequestFullscreen;
    //rfs.call(el);
}


function keyhandler(e) {
    switch (e.key) {
    case "q":
        send_command({score_adj: [-1, 0] });
        break;
    case "w":
        send_command({score_adj: [+1, 0]});
        break;
    case "i":
        send_command({score_adj: [0, +1]});
        break;
    case "o":
        send_command({score_adj: [0, -1]});
        break;
    case "b":
        send_command({start_jam: null }); // TODO: start_jam with jam #
        break;
    case " ":
        send_command({stop_jam: null }); // TODO: stop_jam with jam #
        break;
    case "t":
        send_command({official_timeout: null });
        break;
    }
}
// attach a key handler to body
document.addEventListener('keydown', keyhandler);
document.addEventListener('click', clickhandler);

window.setInterval(function(f) { updater(); }, 500);
