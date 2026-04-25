FROM mcr.microsoft.com/dotnet/sdk:8.0

WORKDIR /workspace/client

COPY client/Client.csproj ./
RUN dotnet restore ./Client.csproj

COPY client/*.cs ./
COPY client/native ./native

RUN dotnet publish -c Release -o /workspace/client/out --no-restore

ENV LD_LIBRARY_PATH=/workspace/client/native
CMD ["dotnet", "/workspace/client/out/Client.dll"]
