/* eslint-disable @typescript-eslint/no-require-imports */
const { expect } = require('chai');
const { exec } = require('child_process');
const castlingTestCases = require('../test/castling.json');

function runTests(testCases) {
  const enginePath = '"./engine/target/debug/BitboardX.exe"';
  describe(testCases.name, () => {
    testCases.tests.forEach(testCase => {
      it(testCase.description, (done) => {
        const child = exec(`${enginePath}`, (err, stdout, stderr) => {
          if (err) {
            expect.fail(`Error: ${err}`);
            return done(err);
          }

          const isInvalidMove = stderr.startsWith('Error: Invalid move');
          expect(testCase.expect).to.equal(!isInvalidMove);
          done(err);
        });
        child.stdin.write(`${testCase.command}\n`);
        child.stdin.end();
      });
    });
  });
}

runTests(castlingTestCases);
