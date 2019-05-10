var Chessground = require('chessground').Chessground;

var ground = Chessground(document.getElementById('chessboard'), {
    fen: '8/N7/8/8/8/k7/NN6/K7',
    movable: {
        free: true,
        color: 'both',
        animation: {
            enabled: true,
            duration: 200
        }
    },
    drawable: {
        enabled: false
    },
    events: {
        change: update
    }
});


var target_field = "a1";

var target = $('#target');
var fen = $('#fen');
var move = $('#move');
var move_checkbox = $('#move :checkbox');
var mate = $('#mate');


set_target_field('g1');
target.val('g1');
update();


move_checkbox.change(function () {
    if (this.checked) {
        ground.set({
            turnColor: 'white'
        });
    } else {
        ground.set({
            turnColor: 'black'
        });
    }
    update();
});


fen.keyup(function (event) {
    if (event.keyCode === 13) {
        ground.set({
            fen: fen.val()
        });
        update()
    }
});

target.keyup(function (event) {
    if (event.keyCode === 13) {
        if (target.val().length === 2 && target.val()[0] >= 'a' && target.val()[0] <= 'h' && target.val()[1] >= '1' && target.val()[1] <= '8')
            set_target_field(target.val());
    }
});



function set_target_field(key) {
    target_field = key;
    update();
}

function update() {
    console.log(ground.state);
    move_checkbox.prop('checked', ground.state.turnColor === 'white');
    fen.val(ground.getFen());
    ground.setShapes([{ orig: target_field, brush: 'red' }]);
    evaluate();
}

function evaluate() {
    $.ajax({
        type: "POST",
        url: "/eval",
        contentType: "application/json",
        data: JSON.stringify({ fen: ground.getFen() + ' ' + ground.state.turnColor[0] + ' - -', target: target_field }),
        success: function (response) {
            if (response.mate_in < 0) {
                mate.text('DRAW')
            }
            else {
                mate.text(Math.trunc(response.mate_in / 2) + ' (' + response.mate_in + ')')
            }
            console.log(response);
            var shapes = response.best_moves.map(function (x) {
                return {
                    orig: x[0],
                    dest: x[1],
                    brush: "green"
                };
            });
            shapes.push({ orig: target_field, brush: 'red' });
            ground.setShapes(shapes );
        },
        error: function (xhr, ajaxOptions, thrownError) {
            mate.text('-')
        }
    });
}