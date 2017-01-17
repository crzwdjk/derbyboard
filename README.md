# derbyboard: a fast roller derby scoreboard

Derbyboard is a roller derby scoreboard written in Rust using the Rocket
web framework. It aims to have high performance and low overhead and thus
be usable on even relatively low-powered hardware. It also aims to allow
for third-party frontends to interface with it easily using a standard
protocol based on DerbyJSON. Note that DerbyJSON support is not currently
implemented, but it's definitely a future goal.

This code is still very much prototype-quality and has many missing and
user-hostile features. Patches and pull requests are, of course, welcome.
