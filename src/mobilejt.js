function send_command(update) {
    console.log(update);
    var xhr = new XMLHttpRequest();
    // TOeDO: debounce enable
    xhr.open('POST', "/score/update");
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onreadystatechange = function () {
        if (xhr.readyState === XMLHttpRequest.DONE) {
            if (xhr.status !== 200) {
                // TODO: handle error
            }
            // TODO: debounce disable
        }
    };
    xhr.send(JSON.stringify(update));
}

function setInner(id, content) {
    document.getElementById(id).innerText = content;
}
function disable(id, yesno) {
    document.getElementById(id).disabled = yesno;
}
function addclick(id, handler) {
    document.getElementById(id).addEventListener('click', handler);
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
    var label = 'none';
    var clock = '0:00';
    for (var label in data) {
        switch (label) {
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
        }
    }
    setInner('clocklabel', label);
    setInner('clocktime', format_time(clock.secs));
}

function updateScore(data) {
    setInner('periodnum', 'Period ' + data.gameclock[0]);
    setInner('periodtime', format_time(data.gameclock[1].secs));
    updateClock(data.activeclock);
    setInner('team1tos', data.timeouts[0]);
    setInner('team1ors', data.reviews[0]);
    setInner('team2tos', data.timeouts[1]);
    setInner('team2ors', data.reviews[1]);
    disable('startjam', data.activeclock.jam);
    disable('stopjam', !data.activeclock.jam);
    disable('team1to', data.timeouts[0] < 1);
    disable('team1or', data.reviews[0] < 1);
    disable('team2to', data.timeouts[1] < 1);
    disable('team2or', data.reviews[1] < 1);
    if (data.activeclock.review) {
        // show OR retained/lost buttons
    }
}
/* The one piece of somewhat interesting state:
   In an OR, the command buttons change to Lost/Retained */

function update() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', "/score/update");
    xhr.onreadystatechange = function () {
        if(xhr.readyState == XMLHttpRequest.DONE && xhr.status == 200) {
            updateScore(JSON.parse(xhr.responseText));
        }
    }
    xhr.send();
}

function init() {
    addclick('startjam', function () { send_command({start_jam: null})} );
    addclick('stopjam', function () { send_command({stop_jam: null})} );
    addclick('timeoutbtn', function () {
        send_command({official_timeout: null})});
    addclick('team1to', function () { send_command({team_timeout: 'Home'})});
    addclick('team1or', function () { send_command({official_review: 'Home'})});
    addclick('team2to', function () { send_command({team_timeout: 'Away'})});
    addclick('team2or', function () { send_command({official_review: 'Away'})});
    window.setInterval(function () { update() }, 500);
}

init()
