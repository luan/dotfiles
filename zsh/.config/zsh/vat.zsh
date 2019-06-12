# VAT workstation

export PATH=$GOPATH/src/code.cloudfoundry.org/cli/out:$PATH
export PATH=$PATH:$HOME/workspace/capi-workspace/scripts
export TARGET_V7=true
export DB=postgres

if [[ -f $HOME/deployments/vbox/creds.yml ]]; then
  export BOSH_ENVIRONMENT=vbox
  export BOSH_CLIENT=admin
  export BOSH_CLIENT_SECRET=$(bosh int ~/deployments/vbox/creds.yml --path /admin_password)
fi

target_bosh() {
  eval "$(_target_bosh $@)"
}

claim_bosh_lite() {
  eval "$(_claim_bosh_lite $@)"
}

int() {
  eval "$(_int $@)"
}

compdef "_files -W $HOME/workspace/cli-pools/bosh-lites/claimed" target_bosh unclaim_bosh_lite
