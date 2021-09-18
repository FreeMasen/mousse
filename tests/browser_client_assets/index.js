
(function() {
    console.log('starting sse test');
    let main = document.getElementById('main');
    let list;
    let sse = new EventSource("/sse");
    let msg_ct = 0;
    sse.onopen = function (...args) {
        console.log('open', args);
        list = document.createElement('ul');
        list.id = 'list';
        main.appendChild(list);
    }
    sse.onmessage = function (msg) {
        console.log('message', msg);
        if (!list) {
            throw new Error('message before open');
        }
        let li = document.createElement('li');
        li.textContent = msg.data;
        li.id = `message-${msg_ct}`
        list.appendChild(li);
        msg_ct += 1;
    }
    sse.onerror = function (err) {
        console.error('error', err.toString());
    }
})()