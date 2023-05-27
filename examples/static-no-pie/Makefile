AS = as
LD = ld

SRC = hello.s
OBJ = hello.o

OUT = a.out

all: $(OUT)

$(OUT): $(OBJ)
	$(LD) $(OBJ) -o $(OUT)

$(OBJ): $(SRC)
	$(AS) $(SRC) -o $(OBJ)

clean:
	rm -f $(OBJ) $(OUT)
