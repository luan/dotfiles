" Called after everything just before setting a default colorscheme
" Configure you own bindings or other preferences. e.g.:

" set nonumber " No line numbers

" let g:gitgutter_signs = 0 " No git gutter signs
" augroup config#after
"   autocmd!
"   autocmd VimEnter * GitGutterDisable
" augroup end
" let g:SignatureEnabledAtStartup = 0 " Do not show marks

" let g:ale_open_list = 1

if filereadable($XDG_RUNTIME_DIR . '/lighttheme')
  set background=light
  let g:lightline.colorscheme = 'solarized'
  colorscheme base16-solarized-light
else
  set background=dark
  let g:lightline.colorscheme = 'material'
endif

au BufNewFile,BufRead /*.rasi setf css

if filereadable(expand('<sfile>:h') . '/post-after.vim')
  runtime user/post-after.vim
endif

autocmd FileType pyst setlocal commentstring=#\ %s
