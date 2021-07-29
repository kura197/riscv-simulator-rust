#/bin/sh

ISA_DIR=$TEST_DIR/isa
SIM=./target/debug/riscv_simulator

for file in `find $ISA_DIR -maxdepth 1 -type f`; do
    if [ -x ""$file -a `echo $file | grep -- "rv32ui-p-"` ]; then
        #output="test $file .. "
        result=`$SIM $file 2> /dev/null`
        #echo $result
        `echo $result | grep -q "a0 : 00000000"`
        if [ $? -ne 0 ]; then
            echo $file
        fi
    fi
done
