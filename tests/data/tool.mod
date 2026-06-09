module github.com/docker/docs

go 1.21

tool example.com/mymodule/cmd/mytool1

tool example.com/mymodule/cmd/mytool2

tool (
    github.com/golangci/golangci-lint/v2/cmd/golangci-lint
    github.com/sqlc-dev/sqlc/cmd/sqlc
    go.temporal.io/sdk/contrib/tools/workflowcheck
)
