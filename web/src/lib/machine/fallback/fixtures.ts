// DEVELOPMENT FALLBACK, see ./index.ts. Machines the fallback is tested with: the binary
// increment, the course exercise A_mul (unary multiplication) and the BB(4) champion.

export const binaryIncrement = `// Adds one to a binary number, most significant bit first.
machine "Binary increment"
kind transducer
input "1011"

Q = {q0, q1, q2}
Σ = {0, 1}
Γ = {0, 1, ␣}
start = q0
F = {q2}

// Walk to the end of the number.
δ(q0, 0) = (q0, 0, R)
δ(q0, 1) = (q0, 1, R)
δ(q0, ␣) = (q1, ␣, L)
// Add one, carrying to the left.
δ(q1, 1) = (q1, 0, L)
δ(q1, 0) = (q2, 1, L)
δ(q1, ␣) = (q2, 1, L)
`;

export const unaryMultiplication = `// Multiplies m and n written in unary as #x...x#x...x#.
// For every x of the first block, copies the second block behind the last #.
machine "Unary multiplication"
kind transducer
input "#xxx#xx#"
output unary x

Q = {q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11}
Σ = {x, #}
Γ = {x, X, #, ␣}
start = q0
F = {q11}

δ(q0, #) = (q1, #, R)
// Mark the next x of the first block, or clean up when none is left.
δ(q1, x) = (q2, X, R)
δ(q1, #) = (q9, ␣, L)
δ(q2, x) = (q2, x, R)
δ(q2, #) = (q3, #, R)
// Mark the next x of the second block and append one x at the end.
δ(q3, X) = (q3, X, R)
δ(q3, x) = (q4, X, R)
δ(q3, #) = (q7, #, L)
δ(q4, x) = (q4, x, R)
δ(q4, X) = (q4, X, R)
δ(q4, #) = (q4, #, R)
δ(q4, ␣) = (q5, x, L)
δ(q5, x) = (q5, x, L)
δ(q5, #) = (q6, #, L)
δ(q6, x) = (q6, x, L)
δ(q6, X) = (q6, X, L)
δ(q6, #) = (q3, #, R)
// Unmark the second block and return to the first.
δ(q7, X) = (q7, x, L)
δ(q7, #) = (q8, #, L)
δ(q8, x) = (q8, x, L)
δ(q8, X) = (q1, X, R)
// Erase both factors, leaving the product.
δ(q9, X) = (q9, ␣, L)
δ(q9, #) = (q10, ␣, R)
δ(q10, ␣) = (q10, ␣, R)
δ(q10, x) = (q10, ␣, R)
δ(q10, #) = (q11, ␣, R)
`;

export const bb4 = `// The four-state busy beaver champion: 107 steps, 13 ones.
machine "BB(4) champion"
kind beaver

Γ = {0, 1}
blank = 0
start = A

δ(A, 0) = (B, 1, R)
δ(A, 1) = (B, 1, L)
δ(B, 0) = (A, 1, L)
δ(B, 1) = (C, 0, L)
δ(C, 0) = (Z, 1, R)
δ(C, 1) = (D, 1, L)
δ(D, 0) = (D, 1, R)
δ(D, 1) = (A, 0, R)
`;
