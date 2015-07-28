var express = require('express');
var path = require('path')
var app = express();

var baseSrc = path.join(__dirname, 'src')

app.use(express.static(path.join(baseSrc, '/html')));
app.use(express.static(path.join(baseSrc, '/image')));
app.use(express.static(path.join(__dirname, '/build')));
app.listen(process.env.PORT || 8080);
