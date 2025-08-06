# TODOs

## Counter is failing. 

It looks like it was off by 2. Expected to read 1312, but got 1310 from all nodes.

```txt
 :workload {:valid? false,
            :errors (#jepsen.history.Op{:index 3889,
                                        :time 30021685237,
                                        :type :ok,
                                        :process 2,
                                        :f :read,
                                        :value 1310,
                                        :final? true}
                     #jepsen.history.Op{:index 3890,
                                        :time 30022154171,
                                        :type :ok,
                                        :process 0,
                                        :f :read,
                                        :value 1310,
                                        :final? true}
                     #jepsen.history.Op{:index 3891,
                                        :time 30022180551,
                                        :type :ok,
                                        :process 1,
                                        :f :read,
                                        :value 1310,
                                        :final? true}),
            :final-reads (1310 1310 1310),
            :acceptable ([1312 1312])},
```

## Broadcast (single-node) is failing

Lost a few messages.

```txt
 :workload {:valid? false,
            :errors (#jepsen.history.Op{:index 3889,
                                        :time 30021685237,
                                        :type :ok,
                                        :process 2,
                                        :f :read,
                                        :value 1310,
                                        :final? true}
                     #jepsen.history.Op{:index 3890,
                                        :time 30022154171,
                                        :type :ok,
                                        :process 0,
                                        :f :read,
                                        :value 1310,
                                        :final? true}
                     #jepsen.history.Op{:index 3891,
                                        :time 30022180551,
                                        :type :ok,
                                        :process 1,
                                        :f :read,
                                        :value 1310,
                                        :final? true}),
            :final-reads (1310 1310 1310),
            :acceptable ([1312 1312])},
```
