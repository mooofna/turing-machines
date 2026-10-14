// Valid, but q2 can never be reached.
machine "unreachable"
F = {q1}

δ(q0, x) = (q0, X, R)
δ(q0, ␣) = (q1, ␣, L)
δ(q2, x) = (q1, x, R)
