.onLoad <- function(libname, pkgname) {
  if (!nzchar(Sys.getenv("RAYON_NUM_THREADS"))) {
    packageStartupMessage("To use parallelization in `ww_area_of_applicability`, set the `RAYON_NUM_THREADS` environment variable to the number of threads that should be used.")
    Sys.setenv(RAYON_NUM_THREADS = 1)
  }
}
