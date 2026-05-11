FROM eclipse-temurin:21-jdk

WORKDIR /workspace/client

COPY client/src ./src
COPY generated/java/src ./generated-src
COPY generated/java/native ./native

RUN find src generated-src -name '*.java' -print0 | xargs -0 javac -d out

CMD ["java", "-Djava.library.path=/workspace/client/native", "-cp", "/workspace/client/out", "com.icsmw.brec.Main"]
