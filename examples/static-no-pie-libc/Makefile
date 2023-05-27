CC=gcc
CFLAGS=-g

OBJ=main.o
OUT=a.out

all: $(OUT)

%.o: %.c
	$(CC) -c -o $@ $< $(CFLAGS)

$(OUT): $(OBJ)
	$(CC) -o $@ $^ $(CFLAGS)

clean:
	rm -f $(OBJ) $(OUT)
