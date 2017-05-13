function jam_command(jam, team, cmd) {
    var xhr = new XMLHttpRequest();
    xhr.open('POST', '/jam/'+jam+'/'+team+'/command');
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onreadystatechange = function () {
        if (xhr.readyState == XMLHttpRequest.DONE && xhr.status == 200) {
            var resp = JSON.parse(xhr.responseText);
            refresh(resp);
        }
    }
    xhr.send(JSON.stringify(cmd));
}


// TODO: undos
function keyhandler(e) {
    console.log(this);
    var jamnum = this.parentElement.getAttribute('data-jam');
    var team = this.parentElement.parentElement.getAttribute('data-team');
    switch (e.key) {
    case 'L': // lead
        jam_command(jamnum, team, {lead: true});
        break;
    case 'N': // pass complete, no lead
        // XXX
        break;
    case 'C':
        jam_command(jamnum, team, {call: true });
        break; // call
    case 'P':
        jam_command(jamnum, team, {starpass: true });
        break;
    case '1': case '2': case '3': case '4': case '5': case '6':
        var points = parseInt(e.key);
        var trip = this.getAttribute('data-trip');
        jam_command(jamnum, team, { "scoring-trip": { trip, points }});
        break;
    case 'Backspace': case 'Delete':
        break;
    case 'Tab':
        // TODO: flip to other team
        break;
    case 'ArrowUp': case 'Up': // go to prev jam,
        if (this.previousElementSibling) {
            this.previousElementSibling.previousElementSibling.focus();
        }
        break;
    case "ArrowDown": case "Down":
        if (this.nextElementSibling) {
            this.nextElementSibling.focus();
        } else {
            
            // TODO: create a new jam
        }
        break;
    }
    // TODO: points '+', '-'
    // TODO: advance jam
}

// TODO: rewrite using only CSS
function showteams() {
    var show1 = document.getElementById('show1').checked;
    var show2 = document.getElementById('show2').checked;
    document.getElementById('Home_score').hidden = show2;
    document.getElementById('Away_score').hidden = show1;
}

function check_child(node, childclass, predicate) {
    var value = predicate ? 'X' : '';
    node.getElementsByClassName(childclass)[0].textContent = value;
}

function toNum(v) { var x = parseInt(v); return Number.isNaN(x) ? 0 : x }

function doRowTotals(row) {
    var gametotal = 0;
    var prevRow = row.previousElementSibling;
    if (prevRow) {
        gametotal = toNum(prevRow.getElementsByClassName('gametotal')[0].textContent);
    }
    var cells = row.getElementsByClassName('scoringtrip');
    var rowtotal = 0;
    for (let i = 0; i < cells.length; i++) {
        rowtotal += toNum(cells[i].textContent);
    }
    row.getElementsByClassName('jamtotal')[0].textContent = rowtotal;
    row.getElementsByClassName('gametotal')[0].textContent = rowtotal + gametotal;
}

function filljam(row, jamnum, jamdata) {
    row.setAttribute('data-jam', jamnum);
    row.getElementsByClassName('jamnum')[0].textContent = jamnum;
    row.getElementsByClassName('jammer')[0].textContent = jamdata.jammer;
    check_child(row, 'lost', jamdata.lost);
    check_child(row, 'lead', jamdata.lead);
    check_child(row, 'call', jamdata.call);
    check_child(row, 'ni', jamdata.jammerpoints.length == 0);

    var col = 0;
    var pointcols = row.getElementsByClassName('scoringtrip');
    jamdata.jammerpoints.forEach(function (trippoints) {
        if (col >= 9) {
            alert("Your jammer is too good!");
            return;
        }
        pointcols[col++].textContent = trippoints;
    });
    doRowTotals(row);
    if (jamdata.starpass) {
        let sprow = row.nextElementSibling || createRow(table);
        sprow.setAttribute('data-jam', jamnum);
        sprow.getElementsByClassName('jamnum')[0].textContent = 'SP';
        sprow.getElementsByClassName('jammer')[0].textContent = jamdata.pivot;
        check_child(sprow, 'ni', jamdata.pivotpoints.length == 0);
        let pointcols = sprow.getElementsByClassName('scoringtrip');
        jamdata.pivotpoints.forEach(function (trippoints) {
            if (col >= 9) {
                alert("Your jammer is too good!");
                return;
            }
            pointcols[col++].textContent = trippoints;
        });
        doRowTotals(sprow);
        return true;
    }
    return false;
}

function createRow(table) {
    var template = document.getElementById('scoreline').content;
    var row = document.importNode(template, true).children[0];
    var cells = row.getElementsByTagName('td');
    for (let i = 0; i < cells.length; i++) {
        cells[i].addEventListener('keydown', keyhandler);
    }
    return table.appendChild(row)
}

function refresh(upd) {
    var rowidx = 0;
    var homerows = document.querySelectorAll('#Home_score>tr');
    var awayrows = document.querySelectorAll('#Away_score>tr');
    var jamnum = 0;
    upd.forEach(function (jam) {
        jamnum++;
        var homerow = homerows[rowidx];
        if (!homerow) homerow = createRow(document.getElementById('Home_score'));
        var awayrow = awayrows[rowidx];
        if (!awayrow) awayrow = createRow(document.getElementById('Away_score'));
        var homesp = filljam(homerow, jamnum, jam[0]);
        var awaysp = filljam(awayrow, jamnum, jam[1]);
        // XXX: actually create the SP* rows
        if (homesp && !awaysp) {
            awayrow.nextElementSibling.getElementsByClassName('jamnum').textContent = 'SP*';
            doRowTotals(awayrow.nextElementSibling);
        }
        if (awaysp && !homesp) {
            homerow.nextElementSibling.getElementsByClassName('jamnum').textContent = 'SP*';
            doRowTotals(homerow.nextElementSibling);
        }
        rowidx++;
        if (homesp || awaysp) rowidx++;
    });
}

function update_cb(team) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', document.location +'/update');
    xhr.onreadystatechange = function () {
        if (xhr.readyState == XMLHttpRequest.DONE && xhr.status == 200) {
            var resp = JSON.parse(xhr.responseText);
            refresh(resp);
        }
    }
    xhr.send();
}

function load() {
    showteams();
    var teambuttons = document.getElementsByName('teams');
    for (let i = 0; i < teambuttons.length; i++) {
        teambuttons[i].addEventListener('change', showteams);
    }
    window.setInterval(update_cb, 1000);
}


load();

// focus model: focus is on a cell, entry auto-advances
// Jammer number, N or L, scoring pass. C for call works anywhere.
// 0-5 completes scoring trip.
// P causes a star pass, drops to next line
// jammer # onclick causes dropdown (should work on mobile)
// down arrow goes to next jam.
