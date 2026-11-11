#include <math.h>
#include <stdio.h>

double dnorm(double, double, double, int);
double dunif(double, double, double, int);
double pexp(double, double, int, int);
double qexp(double, double, int, int);
double pnorm(double, double, double, int, int);
void pnorm_both(double, double *, double *, int, int);
double qnorm(double, double, double, int, int);
double qlogis(double, double, double, int, int);
double plogis(double, double, double, int, int);
double cospi(double);
double logspace_add(double, double);
double gammafn(double);
double lgammafn(double);
double choose(double, double);
double dbinom(double, double, double, int);
double dpois(double, double, int);
double ppois(double, double, int, int);
double dgeom(double, double, int);
double pgeom(double, double, int, int);
double dbeta(double, double, double, int);
double dgamma(double, double, double, int);
double dt(double, double, int);
double df(double, double, double, int);
double pgamma(double, double, double, int, int);
double qgamma(double, double, double, int, int);
double qchisq(double, double, int, int);
double pbeta(double, double, double, int, int);
double qbeta(double, double, double, int, int);
double pt(double, double, int, int);
double qt(double, double, int, int);
double dhyper(double, double, double, double, int);
double phyper(double, double, double, double, int, int);
double qhyper(double, double, double, double, int, int);
double dsignrank(double, double, int);
double psignrank(double, double, int, int);
double qsignrank(double, double, int, int);
double dwilcox(double, double, double, int);
double pwilcox(double, double, double, int, int);
double qwilcox(double, double, double, int, int);
double digamma(double);
double trigamma(double);
double bessel_j(double, double);
double bessel_i(double, double, double);
double bessel_k_ex(double, double, double, double *);
double fround(double, double);
double fprec(double, double);
double pnchisq(double, double, double, int, int);
double qnchisq(double, double, double, int, int);
double pnt(double, double, double, int, int);
double qnt(double, double, double, int, int);
double ptukey(double, double, double, double, int, int);
double qtukey(double, double, double, double, int, int);
void set_seed(unsigned int, unsigned int);
void get_seed(unsigned int *, unsigned int *);
double norm_rand(void);
double rgamma(double, double);
double rhyper(double, double, double);
double rsignrank(double);
double rwilcox(double, double);
void rmultinom(int, double *, int, int *);

int main(void) {
    double dn = dnorm(0.7, 0.0, 1.0, 0);
    double qn = qnorm(0.7, 0.0, 1.0, 1, 0);
    double pn = pnorm(qn, 0.0, 1.0, 1, 0);

    if (fabs(dn - 0.3122539333667613) > 1e-15) {
        fprintf(stderr, "dnorm smoke failed: %.17g\n", dn);
        return 1;
    }
    if (fabs(pn - 0.7) > 1e-15) {
        fprintf(stderr, "pnorm/qnorm smoke failed: %.17g\n", pn);
        return 1;
    }
    double cum = 0.0;
    double ccum = 0.0;
    pnorm_both(0.0, &cum, &ccum, 2, 0);
    if (fabs(cum - 0.5) > 1e-15 || fabs(ccum - 0.5) > 1e-15) {
        fprintf(stderr, "pnorm_both smoke failed\n");
        return 1;
    }
    if (fabs(dunif(0.25, 0.0, 2.0, 0) - 0.5) > 1e-15) {
        fprintf(stderr, "dunif smoke failed\n");
        return 1;
    }
    if (fabs(qexp(pexp(2.0, 3.0, 1, 0), 3.0, 1, 0) - 2.0) > 1e-14) {
        fprintf(stderr, "pexp/qexp smoke failed\n");
        return 1;
    }
    if (fabs(plogis(qlogis(0.7, 0.0, 1.0, 1, 0), 0.0, 1.0, 1, 0) - 0.7) > 1e-15) {
        fprintf(stderr, "plogis/qlogis smoke failed\n");
        return 1;
    }
    if (cospi(0.5) != 0.0) {
        fprintf(stderr, "cospi smoke failed\n");
        return 1;
    }
    if (fabs(logspace_add(2.0, 3.0) - log(exp(2.0) + exp(3.0))) > 1e-15) {
        fprintf(stderr, "logspace_add smoke failed\n");
        return 1;
    }
    if (fabs(gammafn(5.0) - 24.0) > 1e-12 || fabs(lgammafn(5.0) - log(24.0)) > 1e-12) {
        fprintf(stderr, "gamma smoke failed\n");
        return 1;
    }
    if (fabs(choose(10.0, 3.0) - 120.0) > 1e-12) {
        fprintf(stderr, "choose smoke failed\n");
        return 1;
    }
    if (fabs(dbinom(3.0, 10.0, 0.2, 0) - 0.201326592) > 1e-12) {
        fprintf(stderr, "dbinom smoke failed\n");
        return 1;
    }
    if (fabs(dpois(3.0, 2.0, 0) - 0.1804470443154836) > 1e-12) {
        fprintf(stderr, "dpois smoke failed\n");
        return 1;
    }
    if (fabs(ppois(3.0, 2.0, 1, 0) - 0.857123460498547) > 1e-12) {
        fprintf(stderr, "ppois smoke failed\n");
        return 1;
    }
    if (fabs(dgeom(3.0, 0.25, 0) - 0.10546875) > 1e-15 ||
        fabs(pgeom(3.0, 0.25, 1, 0) - 0.68359375) > 1e-15) {
        fprintf(stderr, "geom smoke failed\n");
        return 1;
    }
    if (fabs(dbeta(0.3, 2.0, 3.0, 0) - 1.764) > 1e-12) {
        fprintf(stderr, "dbeta smoke failed\n");
        return 1;
    }
    if (fabs(dgamma(2.0, 3.0, 2.0, 0) - 0.09196986029286064) > 1e-12) {
        fprintf(stderr, "dgamma smoke failed\n");
        return 1;
    }
    if (fabs(dt(1.0, 10.0, 0) - 0.230361989229139) > 1e-10) {
        fprintf(stderr, "dt smoke failed\n");
        return 1;
    }
    if (fabs(df(1.0, 5.0, 10.0, 0) - 0.4954797834866395) > 1e-10) {
        fprintf(stderr, "df smoke failed\n");
        return 1;
    }
    if (fabs(qgamma(pgamma(2.0, 1.0, 3.0, 1, 0), 1.0, 3.0, 1, 0) - 2.0) > 1e-10) {
        fprintf(stderr, "pgamma/qgamma smoke failed\n");
        return 1;
    }
    if (fabs(qbeta(pbeta(0.3, 2.0, 3.0, 1, 0), 2.0, 3.0, 1, 0) - 0.3) > 1e-10) {
        fprintf(stderr, "pbeta/qbeta smoke failed\n");
        return 1;
    }
    if (fabs(qt(pt(1.0, 1.0, 1, 0), 1.0, 1, 0) - 1.0) > 1e-10) {
        fprintf(stderr, "pt/qt smoke failed\n");
        return 1;
    }
    if (fabs(dhyper(2.0, 5.0, 7.0, 4.0, 0) - 420.0 / 990.0) > 1e-12 ||
        fabs(phyper(2.0, 5.0, 7.0, 4.0, 1, 0) - 420.0 / 495.0) > 1e-12 ||
        qhyper(0.5, 5.0, 7.0, 4.0, 1, 0) != 2.0) {
        fprintf(stderr, "hypergeometric smoke failed\n");
        return 1;
    }
    if (fabs(dsignrank(3.0, 3.0, 0) - 0.25) > 1e-15 ||
        fabs(psignrank(3.0, 3.0, 1, 0) - 0.625) > 1e-15 ||
        qsignrank(0.625, 3.0, 1, 0) != 3.0) {
        fprintf(stderr, "signrank smoke failed\n");
        return 1;
    }
    if (fabs(dwilcox(3.0, 2.0, 3.0, 0) - 0.2) > 1e-15 ||
        fabs(pwilcox(3.0, 2.0, 3.0, 1, 0) - 0.6) > 1e-15 ||
        qwilcox(0.6, 2.0, 3.0, 1, 0) != 3.0) {
        fprintf(stderr, "wilcox smoke failed\n");
        return 1;
    }
    if (fround(2.5, 0.0) != 2.0 || fprec(1234.0, 2.0) != 1200.0) {
        fprintf(stderr, "rounding smoke failed\n");
        return 1;
    }
    if (fabs(digamma(1.0) + 0.5772156649015329) > 1e-12 ||
        fabs(trigamma(1.0) - 1.6449340668482264) > 1e-10) {
        fprintf(stderr, "polygamma smoke failed\n");
        return 1;
    }
    double work = 0.0;
    double bk = bessel_k_ex(2.0, 0.5, 0.0, &work);
    if (fabs(bessel_j(1.0, 0.0) - 0.7651976865579666) > 1e-12 ||
        fabs(bessel_i(1.0, 0.0, 0.0) - 1.2660658777520082) > 1e-12 ||
        !isfinite(bk) || work != bk) {
        fprintf(stderr, "bessel smoke failed\n");
        return 1;
    }
    if (fabs(pnchisq(2.0, 4.0, 0.0, 1, 0) - 0.2642411176571153) > 1e-10 ||
        fabs(qnchisq(0.5, 4.0, 0.0, 1, 0) - qchisq(0.5, 4.0, 1, 0)) > 1e-10 ||
        fabs(qnt(pnt(1.0, 10.0, 0.0, 1, 0), 10.0, 0.0, 1, 0) - 1.0) > 1e-10 ||
        fabs(qtukey(ptukey(2.0, 1.0, 3.0, 10.0, 1, 0), 1.0, 3.0, 10.0, 1, 0) - 2.0) > 1e-8) {
        fprintf(stderr, "noncentral/tukey smoke failed\n");
        return 1;
    }

    set_seed(123, 456);
    unsigned int seed1 = 0;
    unsigned int seed2 = 0;
    get_seed(&seed1, &seed2);
    if (seed1 != 123 || seed2 != 456) {
        fprintf(stderr, "get_seed smoke failed: %u %u\n", seed1, seed2);
        return 1;
    }

    double z = norm_rand();
    if (!isfinite(z)) {
        fprintf(stderr, "norm_rand smoke failed: %.17g\n", z);
        return 1;
    }
    double g = rgamma(2.0, 3.0);
    if (!isfinite(g) || g < 0) {
        fprintf(stderr, "rgamma smoke failed: %.17g\n", g);
        return 1;
    }
    if (!isfinite(rhyper(5.0, 7.0, 4.0)) ||
        !isfinite(rsignrank(5.0)) ||
        !isfinite(rwilcox(3.0, 4.0))) {
        fprintf(stderr, "rank rng smoke failed\n");
        return 1;
    }
    double probs[3] = {0.2, 0.3, 0.5};
    int counts[3] = {0, 0, 0};
    rmultinom(12, probs, 3, counts);
    if (counts[0] + counts[1] + counts[2] != 12) {
        fprintf(stderr, "rmultinom smoke failed\n");
        return 1;
    }

    printf("dnorm(0.7) = %.12g\n", dn);
    printf("qnorm(0.7) = %.12g\n", qn);
    printf("pnorm(qnorm(0.7)) = %.12g\n", pn);
    printf("one normal = %.12g\n", z);
    return 0;
}
