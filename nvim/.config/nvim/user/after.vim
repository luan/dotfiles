" Called after everything just before setting a default colorscheme
" Configure you own bindings or other preferences. e.g.:

set nonumber " No line numbers
" let g:gitgutter_signs = 0 " No git gutter signs
augroup config#after
  autocmd!
  autocmd VimEnter * GitGutterDisable
augroup end
let g:SignatureEnabledAtStartup = 0 " Do not show marks

" let g:ale_open_list = 1

if filereadable($XDG_RUNTIME_DIR . '/lighttheme')
  set background=light
  let g:lightline.colorscheme = 'solarized'
  colorscheme base16-solarized-light
else
  set background=dark
  let g:lightline.colorscheme = 'material'
endif

function! CFCLIIntegrationTransform(cmd) abort
  let l:cmd = a:cmd

  if $TARGET_V7 ==# 'true' && l:cmd =~# 'ginkgo'
    let l:cmd = substitute(l:cmd, 'ginkgo', 'ginkgo --tags V7', 1)
  endif

  if getcwd() =~# 'cli' && l:cmd =~# 'integration'
    return 'make build && '.l:cmd
  endif

  return l:cmd
endfunction

let g:test#custom_transformations = { 'cfcli': function('CFCLIIntegrationTransform') }
let g:test#transformation = 'cfcli'

au BufNewFile,BufRead /*.rasi setf css

if filereadable(expand('<sfile>:h') . '/post-after.vim')
  runtime user/post-after.vim
endif

