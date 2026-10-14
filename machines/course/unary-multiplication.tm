// A_mul: multiplication of two numbers in unary.
//
// TheoLog 2024, exercise sheet 1, task 3. The input #x^m#x^n# becomes
// x^(m × n). For every x of the first factor (marked as X), the machine
// copies the second factor x by x (also marked as X) behind the third #,
// then restores the second factor. When the first factor is used up, it
// erases everything left of the product.
//
// This is the machine of the 2024 simulator. States q0 to q9 are
// unchanged; the cleanup q10 to q12 is new, because the original shifted
// the product to the left and stopped with N moves.
machine "A_mul"
input "#xxx#xx#"
output unary x

Q = {q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11, q12}
Σ = {x, #}
Γ = {x, X, #, ␣}
start = q0
F = {q12}

δ(q0, #) = (q1, #, R)  // skip the first separator

// Take the next x of the first factor, or clean up if there is none.
δ(q1, x) = (q2, X, R)
δ(q1, #) = (q10, ␣, L)

δ(q2, x) = (q2, x, R)  // skip the rest of the first factor
δ(q2, #) = (q3, #, R)

// Take the next x of the second factor, or restore it if there is none.
δ(q3, x) = (q4, X, R)
δ(q3, #) = (q8, #, L)

// Append one x to the product.
δ(q4, x) = (q4, x, R)
δ(q4, #) = (q5, #, R)
δ(q5, x) = (q5, x, R)
δ(q5, ␣) = (q6, x, L)

// Back to the last X of the second factor.
δ(q6, x) = (q6, x, L)
δ(q6, #) = (q7, #, L)
δ(q7, x) = (q7, x, L)
δ(q7, X) = (q3, X, R)

// Restore the second factor, then go back to the last X of the first.
δ(q8, X) = (q8, x, L)
δ(q8, #) = (q9, #, L)
δ(q9, x) = (q9, x, L)
δ(q9, X) = (q1, X, R)

// Erase the first factor, the second factor and the separators.
δ(q10, X) = (q10, ␣, L)
δ(q10, #) = (q11, ␣, R)
δ(q11, {x, ␣}) = (q11, ␣, R)
δ(q11, #) = (q12, ␣, R)    // the product starts right here
