goto() {
  local p
  local f

  for p in `echo $GOPATH | tr ':' '\n'`; do
    f=`find ${p}/src -maxdepth 3 -type d | grep ${1} | head -n 1`
    if [ -n "$f" ]; then
      cd $f
      return
    fi
  done
}
