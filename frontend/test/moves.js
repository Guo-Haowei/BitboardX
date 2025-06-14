/* eslint-disable @typescript-eslint/no-require-imports */
const { expect } = require('chai');
const { exec } = require('child_process');
const { MOVES_DATA } = require('./moves_data.js');

const enginePath = '"../target/debug/BitboardX.exe"';

function testMove(testCase, fen) {
  fen = fen || testCase.fen;
  it(testCase.description, (done) => {
    const child = exec(`${enginePath}`, (err, stdout, stderr) => {
      if (err) {
        throw new Error(`Error executing engine: ${err.message}`);
      }

      const isInvalidMove = stderr.startsWith('Error: Invalid move');
      expect(testCase.expect).to.equal(!isInvalidMove);
      // console.log(`error: ${isInvalidMove ? stderr : 'No error'}`);
      // console.log(`stdout: ${stdout}`);
      // console.log(`stderr: ${stderr}`);
      done(err);
    });
    child.stdin.write(`position fen ${fen} moves ${testCase.moves}\n`);
    child.stdin.end();
  });
}

for (const [key, value] of Object.entries(MOVES_DATA)) {
  describe(key, () => {
    value.forEach(testCase => {
      if (Array.isArray(testCase)) {
        const fen = testCase.shift();
        testCase.forEach(tc => {
          testMove(tc, fen);
        });
      } else if (typeof testCase === 'object') {
        testMove(testCase);
      }
    });
  });
}
