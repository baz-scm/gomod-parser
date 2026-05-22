module github.com/example/ignore

go 1.24

ignore ./build

ignore (
    ./testdata
    ./vendor/temp
    ./node_modules
)
