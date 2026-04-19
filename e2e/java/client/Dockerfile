FROM eclipse-temurin:21-jdk

WORKDIR /workspace/client

COPY client/src ./src
COPY client/native ./native

RUN find src -name '*.java' -print0 | xargs -0 javac -d out

CMD ["java", "-Djava.library.path=/workspace/client/native", "-cp", "/workspace/client/out", "com.icsmw.brec.Main"]
