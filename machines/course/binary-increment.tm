// Binary increment: adds one to a binary number written with the most significant bit first.
// The head runs to the right end of the input, then carries to the left.
machine "Binary increment"
kind transducer
input "1011"
output binary msb

Q = {right, carry, done}
Σ = {0, 1}
Γ = {0, 1, ␣}
start = right
F = {done}

δ(right, 0) = (right, 0, R)
δ(right, 1) = (right, 1, R)
δ(right, ␣) = (carry, ␣, L)
δ(carry, 1) = (carry, 0, L)
δ(carry, 0) = (done, 1, L)
δ(carry, ␣) = (done, 1, L)
