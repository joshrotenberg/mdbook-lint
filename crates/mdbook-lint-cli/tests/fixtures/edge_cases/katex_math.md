# KaTeX Comprehensive Test File

This file contains a comprehensive set of KaTeX expressions for testing markdown/mdbook rendering.

## Inline Math Basics

Simple inline: $x + y = z$

Variables: $a$, $b$, $c$, $x_1$, $x_2$, $y_{max}$, $z^2$, $w^{n+1}$

Subscripts and superscripts combined: $x_i^2$, $a_{i,j}^{k,l}$, ${}_nC_r$

## Display Math Basics

$$x + y = z$$

$$a^2 + b^2 = c^2$$

## Greek Letters

### Lowercase

$$\alpha \beta \gamma \delta \epsilon \varepsilon \zeta \eta \theta \vartheta \iota \kappa \varkappa \lambda \mu \nu \xi \omicron \pi \varpi \rho \varrho \sigma \varsigma \tau \upsilon \phi \varphi \chi \psi \omega$$

### Uppercase

$$\Gamma \Delta \Theta \Lambda \Xi \Pi \Sigma \Upsilon \Phi \Psi \Omega$$

## Hebrew Letters

$$\aleph \beth \gimel \daleth$$

## Arrows

### Standard Arrows

$$\leftarrow \rightarrow \uparrow \downarrow \leftrightarrow \updownarrow$$

$$\Leftarrow \Rightarrow \Uparrow \Downarrow \Leftrightarrow \Updownarrow$$

$$\longleftarrow \longrightarrow \longleftrightarrow$$

$$\Longleftarrow \Longrightarrow \Longleftrightarrow$$

### Special Arrows

$$\mapsto \longmapsto \hookrightarrow \hookleftarrow$$

$$\nearrow \searrow \swarrow \nwarrow$$

$$\rightharpoonup \rightharpoondown \leftharpoonup \leftharpoondown \rightleftharpoons$$

$$\leadsto \dashrightarrow \dashleftarrow$$

### Extensible Arrows

$$\xrightarrow{abc} \xleftarrow{def} \xRightarrow{ghi} \xLeftarrow{jkl}$$

$$\xrightarrow[below]{above} \xleftarrow[below]{above}$$

$$\xlongequal{text}$$

## Accents

$$\hat{a} \bar{b} \dot{c} \ddot{d} \tilde{e} \vec{f} \check{g} \breve{h} \acute{i} \grave{j}$$

$$\widehat{abc} \widetilde{xyz} \overline{abc} \underline{def}$$

$$\overrightarrow{AB} \overleftarrow{CD} \overleftrightarrow{EF}$$

$$\underrightarrow{GH} \underleftarrow{IJ} \underleftrightarrow{KL}$$

$$\overbrace{a + b + c}^{n} \underbrace{x + y + z}_{m}$$

$$\overgroup{ABC} \undergroup{XYZ}$$

## Delimiters

### Basic Delimiters

$$( ) [ ] \{ \} \langle \rangle | \| / \backslash$$

$$\lvert x \rvert \lVert y \rVert$$

$$\lceil x \rceil \lfloor y \rfloor$$

$$\ulcorner \urcorner \llcorner \lrcorner$$

### Auto-Sizing Delimiters

$$\left( \frac{a}{b} \right)$$

$$\left[ \sum_{i=1}^{n} x_i \right]$$

$$\left\{ \prod_{j=1}^{m} y_j \right\}$$

$$\left\langle \int_0^\infty f(x) dx \right\rangle$$

$$\left| \frac{x}{y} \right|$$

$$\left\| \vec{v} \right\|$$

### Manual Sizing

$$\bigl( \Bigl( \biggl( \Biggl($$

$$\bigr) \Bigr) \biggr) \Biggr)$$

$$\big[ \Big[ \bigg[ \Bigg[$$

$$\big\{ \Big\{ \bigg\{ \Bigg\{$$

$$\big\langle \Big\langle \bigg\langle \Bigg\langle$$

### Mixed Delimiters

$$\left( x \right]$$

$$\left. \frac{dx}{dy} \right|_{y=0}$$

## Fractions

$$\frac{a}{b}$$

$$\dfrac{a}{b}$$

$$\tfrac{a}{b}$$

$$\cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{x}}}$$

$$\frac{\frac{a}{b}}{\frac{c}{d}}$$

$$a/b$$

$$^a/_b$$

$$\genfrac{(}{)}{2pt}{1}{a}{b}$$

## Binomials

$$\binom{n}{k}$$

$$\dbinom{n}{k}$$

$$\tbinom{n}{k}$$

$${n \choose k}$$

$${n \brack k}$$

$${n \brace k}$$

## Roots

$$\sqrt{x}$$

$$\sqrt{x^2 + y^2}$$

$$\sqrt[3]{x}$$

$$\sqrt[n]{a^n}$$

$$\sqrt{\sqrt{\sqrt{x}}}$$

## Sums, Products, Integrals

### Sums

$$\sum_{i=1}^{n} x_i$$

$$\sum\limits_{i=1}^{n} x_i$$

$$\sum\nolimits_{i=1}^{n} x_i$$

$$\displaystyle\sum_{i=1}^{n} x_i$$

### Products

$$\prod_{i=1}^{n} x_i$$

$$\coprod_{i=1}^{n} x_i$$

### Integrals

$$\int_a^b f(x) dx$$

$$\iint_D f(x,y) dA$$

$$\iiint_V f(x,y,z) dV$$

$$\oint_C \vec{F} \cdot d\vec{r}$$

$$\oiint_S \vec{F} \cdot d\vec{S}$$

$$\intop \smallint$$

### Other Big Operators

$$\bigcup_{i=1}^{n} A_i$$

$$\bigcap_{i=1}^{n} A_i$$

$$\bigoplus_{i=1}^{n} V_i$$

$$\bigotimes_{i=1}^{n} W_i$$

$$\bigvee_{i=1}^{n} P_i$$

$$\bigwedge_{i=1}^{n} Q_i$$

$$\bigsqcup_{i=1}^{n} S_i$$

$$\biguplus_{i=1}^{n} T_i$$

$$\bigodot_{i=1}^{n} U_i$$

## Limits

$$\lim_{x \to \infty} f(x)$$

$$\lim\limits_{x \to 0} \frac{\sin x}{x} = 1$$

$$\varlimsup_{n \to \infty} a_n$$

$$\varliminf_{n \to \infty} a_n$$

$$\limsup_{n \to \infty} a_n$$

$$\liminf_{n \to \infty} a_n$$

## Binary Operators

### Arithmetic

$$+ - \pm \mp \times \div \cdot \ast \star$$

### Set Operations

$$\cup \cap \sqcup \sqcap \setminus \smallsetminus$$

$$\uplus \vee \wedge \oplus \ominus \otimes \oslash \odot$$

$$\circ \bullet \diamond \triangleleft \triangleright$$

$$\bigtriangleup \bigtriangledown \lhd \rhd \unlhd \unrhd$$

$$\amalg \wr \dagger \ddagger$$

## Relations

### Equality and Inequality

$$= \ne \neq \equiv \not\equiv$$

$$< > \le \leq \ge \geq$$

$$\ll \gg \lll \ggg$$

$$\leqslant \geqslant \lessgtr \gtrless$$

### Similarity and Approximation

$$\sim \simeq \approx \cong \backsim$$

$$\approxeq \thicksim \thickapprox \backsimeq$$

### Ordering

$$\prec \succ \preceq \succeq$$

$$\preccurlyeq \succcurlyeq \curlyeqprec \curlyeqsucc$$

### Set Relations

$$\subset \supset \subseteq \supseteq$$

$$\subsetneq \supsetneq \subseteqq \supseteqq$$

$$\sqsubset \sqsupset \sqsubseteq \sqsupseteq$$

$$\in \ni \notin \owns$$

### Other Relations

$$\propto \varpropto \parallel \nparallel \perp$$

$$\vdash \dashv \models \vDash \Vdash \Vvdash$$

$$\mid \nmid \shortmid \nshortmid$$

$$\smile \frown \asymp \bowtie$$

## Negated Relations

$$\not= \not< \not> \not\le \not\ge$$

$$\not\equiv \not\sim \not\simeq \not\approx$$

$$\not\subset \not\supset \not\subseteq \not\supseteq$$

$$\not\in \not\ni$$

$$\nless \ngtr \nleq \ngeq \nleqslant \ngeqslant$$

$$\nsubseteq \nsupseteq \nsubseteqq \nsupseteqq$$

$$\nprec \nsucc \npreceq \nsucceq$$

$$\ntriangleleft \ntriangleright \ntrianglelefteq \ntrianglerighteq$$

## Logic and Set Theory

$$\forall \exists \nexists \neg \lnot$$

$$\land \lor \And$$

$$\implies \impliedby \iff$$

$$\therefore \because$$

$$\emptyset \varnothing \complement$$

$$\top \bot$$

$$\vdash \dashv \vDash \models \Vdash$$

## Dots

$$\ldots \cdots \vdots \ddots \iddots$$

$$a_1, a_2, \ldots, a_n$$

$$a_1 + a_2 + \cdots + a_n$$

$$\begin{matrix} 1 & 2 & 3 \\ \vdots & \ddots & \vdots \\ 7 & 8 & 9 \end{matrix}$$

## Spacing

No space: $ab$

Thin space: $a\,b$

Medium space: $a\:b$

Thick space: $a\;b$

Quad space: $a\quad b$

Double quad: $a\qquad b$

Negative thin: $a\!b$

Custom: $a\hspace{1em}b$

Phantom: $a\phantom{xyz}b$

Horizontal phantom: $a\hphantom{xyz}b$

Vertical phantom: $a\vphantom{xyz}b$

## Font Styles

$$\mathrm{Roman}$$

$$\mathit{Italic}$$

$$\mathbf{Bold}$$

$$\mathsf{Sans Serif}$$

$$\mathtt{Monospace}$$

$$\mathcal{CALLIGRAPHIC}$$

$$\mathfrak{Fraktur}$$

$$\mathbb{BLACKBOARD}$$

$$\mathscr{SCRIPT}$$

$$\mathbfit{Bold Italic}$$

$$\text{Normal text in math}$$

$$\textbf{Bold text}$$

$$\textit{Italic text}$$

$$\textsf{Sans text}$$

$$\texttt{Mono text}$$

$$\textrm{Roman text}$$

$$\boldsymbol{\alpha + \beta}$$

Common blackboard: $\mathbb{N}, \mathbb{Z}, \mathbb{Q}, \mathbb{R}, \mathbb{C}, \mathbb{H}$

## Sizing

$${\tiny tiny}$$

$${\scriptsize scriptsize}$$

$${\footnotesize footnotesize}$$

$${\small small}$$

$${\normalsize normalsize}$$

$${\large large}$$

$${\Large Large}$$

$${\LARGE LARGE}$$

$${\huge huge}$$

$${\Huge Huge}$$

## Colors

$$\color{red}{red}$$

$$\color{blue}{blue}$$

$$\color{green}{green}$$

$$\color{orange}{orange}$$

$$\color{purple}{purple}$$

$$\textcolor{teal}{teal text}$$

$$\colorbox{yellow}{boxed}$$

$$\fcolorbox{red}{yellow}{framed}$$

$${\color{red} x} + {\color{blue} y} = {\color{green} z}$$

Hex colors: $\color{#ff6600}{orange}$ and $\color{#0066ff}{blue}$

## Boxing

$$\boxed{E = mc^2}$$

$$\fbox{boxed text}$$

$$\hbox{horizontal box}$$

$$\mbox{math box}$$

## Stacking and Positioning

$$\stackrel{?}{=}$$

$$\overset{n}{\overbrace{a+b+c}}$$

$$\underset{m}{\underbrace{x+y+z}}$$

$$a \atop b$$

$$\substack{i = 1 \\ j = 2}$$

$$\sum_{\substack{0 \le i \le m \\ 0 \le j \le n}} P(i,j)$$

## Matrices and Arrays

### Matrix Environments

$$\begin{matrix} a & b \\ c & d \end{matrix}$$

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix}$$

$$\begin{bmatrix} a & b \\ c & d \end{bmatrix}$$

$$\begin{Bmatrix} a & b \\ c & d \end{Bmatrix}$$

$$\begin{vmatrix} a & b \\ c & d \end{vmatrix}$$

$$\begin{Vmatrix} a & b \\ c & d \end{Vmatrix}$$

### Small Matrices (Inline)

Inline matrix: $\bigl(\begin{smallmatrix} a & b \\ c & d \end{smallmatrix}\bigr)$

### Larger Matrices

$$\begin{pmatrix}
1 & 2 & 3 & 4 \\
5 & 6 & 7 & 8 \\
9 & 10 & 11 & 12 \\
13 & 14 & 15 & 16
\end{pmatrix}$$

### Arrays with Alignment

$$\begin{array}{lcr}
\text{left} & \text{center} & \text{right} \\
l & c & r \\
\hline
a & b & c
\end{array}$$

### Arrays with Lines

$$\left(\begin{array}{c|c}
a & b \\
\hline
c & d
\end{array}\right)$$

$$\begin{array}{|c|c|c|}
\hline
a & b & c \\
\hline
d & e & f \\
\hline
\end{array}$$

## Alignment Environments

### Aligned

$$\begin{aligned}
f(x) &= (x+1)^2 \\
&= x^2 + 2x + 1
\end{aligned}$$

### Align at Multiple Points

$$\begin{alignedat}{2}
10&x + &3&y = 2 \\
3&x + &13&y = 4
\end{alignedat}$$

### Gathered

$$\begin{gathered}
a = b \\
c = d \\
e = f
\end{gathered}$$

### Split

$$\begin{equation}
\begin{split}
a &= b + c \\
&= d + e
\end{split}
\end{equation}$$

## Cases

$$f(x) = \begin{cases}
x^2 & \text{if } x \geq 0 \\
-x^2 & \text{if } x < 0
\end{cases}$$

$$|x| = \begin{cases}
x & x \geq 0 \\
-x & x < 0
\end{cases}$$

$$\begin{rcases}
a &= 1 \\
b &= 2
\end{rcases} \Rightarrow a + b = 3$$

## Equation Numbering (if supported)

$$\begin{equation}
E = mc^2 \tag{1}
\end{equation}$$

$$\begin{equation}
F = ma \tag{Newton}
\end{equation}$$

$$x = y \tag*{[Custom]}$$

## Cancellation

$$\cancel{x}$$

$$\bcancel{x}$$

$$\xcancel{x}$$

$$\sout{strikeout}$$

$$\frac{\cancel{x}}{\cancel{x}} = 1$$

$$\cancelto{0}{x}$$

## Decorations

### Over and Under

$$\overline{AB}$$

$$\underline{XY}$$

$$\overrightarrow{AB}$$

$$\underrightarrow{CD}$$

$$\overleftarrow{EF}$$

$$\underleftarrow{GH}$$

$$\overleftrightarrow{IJ}$$

$$\underleftrightarrow{KL}$$

$$\overbrace{a + b + c}^{\text{sum}}$$

$$\underbrace{x + y + z}_{\text{total}}$$

$$\boxed{a + b = c}$$

### Extensible Symbols

$$\widehat{abc}$$

$$\widetilde{xyz}$$

$$\utilde{abc}$$

## Special Functions

$$\sin \cos \tan \cot \sec \csc$$

$$\arcsin \arccos \arctan$$

$$\sinh \cosh \tanh \coth$$

$$\exp \log \ln \lg$$

$$\det \dim \ker \hom \deg$$

$$\arg \gcd \inf \sup \min \max$$

$$\Pr \lim \limsup \liminf$$

$$\operatorname{custom}(x)$$

$$\operatorname*{argmax}_{x} f(x)$$

## Modular Arithmetic

$$a \bmod b$$

$$a \pmod{n}$$

$$a \pod{n}$$

$$a \equiv b \pmod{m}$$

## Vertical Layout

$$\overset{above}{base}$$

$$\underset{below}{base}$$

$$\stackrel{top}{bottom}$$

$$\atop$$

$$a \above{1pt} b$$

$$a \abovewithdelims [ ] {1pt} b$$

## Physics Notation (if physics extension available)

$$\bra{\phi}$$

$$\ket{\psi}$$

$$\braket{\phi|\psi}$$

$$\Bra{\Phi}$$

$$\Ket{\Psi}$$

## Units (if siunitx-like support)

Some systems support: $\pu{kg.m.s^{-2}}$ or $\SI{9.8}{m/s^2}$

## Chemical Equations (if mhchem extension available)

$$\ce{H2O}$$

$$\ce{CO2}$$

$$\ce{H2SO4}$$

$$\ce{2H2 + O2 -> 2H2O}$$

$$\ce{A <=> B}$$

$$\ce{Cu^2+}$$

$$\ce{SO4^2-}$$

## Miscellaneous Symbols

### Geometry

$$\angle \measuredangle \sphericalangle$$

$$\triangle \square \pentagon \hexagon$$

$$\Diamond \lozenge \blacklozenge$$

$$\bigcirc \bigtriangleup \bigtriangledown$$

### Cards and Music

$$\clubsuit \diamondsuit \heartsuit \spadesuit$$

$$\flat \natural \sharp$$

### Other

$$\infty \partial \nabla \hbar \imath \jmath \ell \Re \Im \wp$$

$$\aleph \beth \gimel$$

$$\eth \Finv \Game$$

$$\degree \checkmark \maltese$$

$$\dag \ddag \S \P \copyright \circledR$$

$$\Box \square \blacksquare$$

$$\triangle \triangledown \blacktriangle \blacktriangledown$$

$$\star \bigstar$$

$$\surd \urcorner \ulcorner \llcorner \lrcorner$$

### Currency

$$\$ \pounds \euro \yen$$

### Astronomical

$$\sun \mercury \venus \earth \mars \jupiter \saturn$$

## Complex Examples

### Quadratic Formula

$$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

### Euler's Identity

$$e^{i\pi} + 1 = 0$$

### Gaussian Integral

$$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$

### Basel Problem

$$\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}$$

### Taylor Series

$$e^x = \sum_{n=0}^{\infty} \frac{x^n}{n!} = 1 + x + \frac{x^2}{2!} + \frac{x^3}{3!} + \cdots$$

$$\sin x = \sum_{n=0}^{\infty} \frac{(-1)^n x^{2n+1}}{(2n+1)!} = x - \frac{x^3}{3!} + \frac{x^5}{5!} - \cdots$$

### Fourier Transform

$$\hat{f}(\xi) = \int_{-\infty}^{\infty} f(x) e^{-2\pi i x \xi} dx$$

### Maxwell's Equations

$$\nabla \cdot \vec{E} = \frac{\rho}{\epsilon_0}$$

$$\nabla \cdot \vec{B} = 0$$

$$\nabla \times \vec{E} = -\frac{\partial \vec{B}}{\partial t}$$

$$\nabla \times \vec{B} = \mu_0 \vec{J} + \mu_0 \epsilon_0 \frac{\partial \vec{E}}{\partial t}$$

### Schrödinger Equation

$$i\hbar \frac{\partial}{\partial t} \Psi(\vec{r}, t) = \hat{H} \Psi(\vec{r}, t)$$

### Einstein Field Equations

$$R_{\mu\nu} - \frac{1}{2} R g_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}$$

### Navier-Stokes Equation

$$\rho \left( \frac{\partial \vec{v}}{\partial t} + \vec{v} \cdot \nabla \vec{v} \right) = -\nabla p + \mu \nabla^2 \vec{v} + \vec{f}$$

### Cauchy-Riemann Equations

$$\frac{\partial u}{\partial x} = \frac{\partial v}{\partial y} \quad \text{and} \quad \frac{\partial u}{\partial y} = -\frac{\partial v}{\partial x}$$

### Stirling's Approximation

$$n! \approx \sqrt{2\pi n} \left( \frac{n}{e} \right)^n$$

### Determinant Expansion

$$\det(A) = \sum_{\sigma \in S_n} \operatorname{sgn}(\sigma) \prod_{i=1}^{n} a_{i,\sigma(i)}$$

### Binomial Theorem

$$(x + y)^n = \sum_{k=0}^{n} \binom{n}{k} x^{n-k} y^k$$

### Residue Theorem

$$\oint_C f(z) dz = 2\pi i \sum_{k=1}^{n} \operatorname{Res}(f, a_k)$$

### Jordan Normal Form

$$J = \begin{pmatrix}
J_1 & 0 & \cdots & 0 \\
0 & J_2 & \cdots & 0 \\
\vdots & \vdots & \ddots & \vdots \\
0 & 0 & \cdots & J_k
\end{pmatrix}$$

where each $J_i$ is a Jordan block:

$$J_i = \begin{pmatrix}
\lambda_i & 1 & 0 & \cdots & 0 \\
0 & \lambda_i & 1 & \cdots & 0 \\
\vdots & \vdots & \ddots & \ddots & \vdots \\
0 & 0 & \cdots & \lambda_i & 1 \\
0 & 0 & \cdots & 0 & \lambda_i
\end{pmatrix}$$

### Continued Fraction

$$\phi = 1 + \cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{1 + \cdots}}}}$$

### Multiline Derivation

$$\begin{aligned}
\nabla \times (\nabla \times \vec{E}) &= \nabla(\nabla \cdot \vec{E}) - \nabla^2 \vec{E} \\
&= -\frac{\partial}{\partial t}(\nabla \times \vec{B}) \\
&= -\mu_0 \epsilon_0 \frac{\partial^2 \vec{E}}{\partial t^2}
\end{aligned}$$

### Commutative Diagram (using array)

$$\begin{array}{ccc}
A & \xrightarrow{f} & B \\
\downarrow{g} & & \downarrow{h} \\
C & \xrightarrow{i} & D
\end{array}$$

## Edge Cases and Special Scenarios

### Empty/Minimal

$${}$$

$$a$$

### Deeply Nested

$$\sqrt{\sqrt{\sqrt{\sqrt{\sqrt{x}}}}}$$

$$\frac{\frac{\frac{\frac{a}{b}}{c}}{d}}{e}$$

### Very Long Expressions

$$a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p + q + r + s + t + u + v + w + x + y + z$$

### Mixed Text and Math

$$\text{The equation } E = mc^2 \text{ is famous.}$$

### Unicode in Math (if supported)

$$α + β = γ$$

### Raw Characters vs Commands

Compare: $\alpha$ vs $α$ (if unicode supported)

### Escaped Characters

$$\$ \% \& \# \_ \{ \}$$

### Line Breaks in Display (if supported)

$$a = b \\ c = d$$

### Multiple Equations

$$x = 1$$
$$y = 2$$
$$z = 3$$

## End of Test File
