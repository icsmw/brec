FROM mcr.microsoft.com/dotnet/sdk:8.0

WORKDIR /workspace/client

COPY client/Client.csproj ./
COPY generated/csharp/Protocol.csproj /workspace/generated/csharp/Protocol.csproj
RUN dotnet restore ./Client.csproj

COPY client/*.cs ./
COPY generated/csharp /workspace/generated/csharp

RUN dotnet publish -c Release -o /workspace/client/out --no-restore

ENV LD_LIBRARY_PATH=/workspace/client/out
CMD ["dotnet", "/workspace/client/out/Client.dll"]
