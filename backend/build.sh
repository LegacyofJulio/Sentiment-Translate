export LIBTORCH=/home/pirson/.libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
cargo watch -x run
